use whisky::*;

pub struct VaultOracleUtxo {
    pub script_utxo: UTxO,
    pub redeemer: String,
    pub datum: String,
    pub script: ProvidedScriptSource,
}

pub struct BurnDepositIntent {
    pub redeemer: String,
    pub burn_amount: i128,
    pub script: ProvidedScriptSource,
}

pub struct IntentUtxo {
    pub script_utxo: UTxO,
    pub redeemer: String,
    pub script: ProvidedScriptSource,
}
pub struct IntentOutput {
    pub output_amount: Vec<Asset>,
    pub address: String,
}

pub struct AppDepositRequest {
    pub redeemer: String,
    pub output_amount: Vec<Asset>,
    pub address: String,
    pub datum: String,
    pub script: ProvidedScriptSource,
}

pub struct LPToken {
    pub redeemer: String,
    pub to_mint_asset: Asset,
    pub script: ProvidedScriptSource,
}

pub async fn batch_process_deposit_intent(
    app_oracle_utxo: &UtxoInput,
    vault_oracle_utxo: &VaultOracleUtxo,
    deposit_intent_to_mint: &BurnDepositIntent,
    intent_utxos: &[IntentUtxo],
    intent_outputs: &[IntentOutput],
    app_deposit_request_to_mint: &AppDepositRequest,
    lp_token_to_mint: &LPToken,
    my_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
) -> Result<String, WError> {
    let mut tx_builder = TxBuilder::new_core();

    let VaultOracleUtxo {
        script_utxo: vault_oracle_script_utxo,
        redeemer: vault_oracle_redeemer,
        datum: vault_oracle_datum,
        script: vault_oracle_script,
    } = vault_oracle_utxo;

    let BurnDepositIntent {
        redeemer: deposit_intent_redeemer,
        script: deposit_intent_script,
        burn_amount,
    } = deposit_intent_to_mint;

    let AppDepositRequest {
        redeemer: app_deposit_request_redeemer,
        output_amount: app_deposit_request_output_amount,
        datum: app_deposit_request_datum,
        address: app_deposit_request_address,
        script: app_deposit_request_script,
    } = app_deposit_request_to_mint;

    let LPToken {
        redeemer: lp_token_redeemer,
        to_mint_asset: lp_token_asset,
        script: lp_token_script,
    } = lp_token_to_mint;

    let deposit_intent_script_hash = get_script_hash(
        &deposit_intent_script.script_cbor,
        deposit_intent_script.language_version.clone(),
    )?;

    let app_deposit_request_script_hash = get_script_hash(
        &app_deposit_request_script.script_cbor,
        app_deposit_request_script.language_version.clone(),
    )?;

    tx_builder
        // burn intents
        .mint_plutus_script_v3()
        .mint(*burn_amount, &deposit_intent_script_hash, "")
        .minting_script(&deposit_intent_script.script_cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(deposit_intent_redeemer.to_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        // mint app deposit request
        .mint_plutus_script_v3()
        .mint(1, &app_deposit_request_script_hash, "")
        .minting_script(&app_deposit_request_script.script_cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(app_deposit_request_redeemer.to_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        .tx_out(
            &app_deposit_request_address,
            app_deposit_request_output_amount,
        )
        .tx_out_inline_datum_value(&WData::JSON(app_deposit_request_datum.to_string()))
        //mint lp token
        .mint_plutus_script_v3()
        .mint(
            lp_token_asset.quantity_i128(),
            &lp_token_asset.policy(),
            &lp_token_asset.name(),
        )
        .minting_script(&lp_token_script.script_cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(lp_token_redeemer.to_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        // app oracle ref input
        .read_only_tx_in_reference(&app_oracle_utxo.tx_hash, app_oracle_utxo.output_index, None)
        .tx_in_inline_datum_present()
        // vault oracle input
        .spending_plutus_script_v3()
        .tx_in(
            &vault_oracle_script_utxo.input.tx_hash,
            vault_oracle_script_utxo.input.output_index,
            &vault_oracle_script_utxo.output.amount,
            &vault_oracle_script_utxo.output.address,
        )
        .tx_in_inline_datum_present()
        .tx_in_redeemer_value(&WRedeemer {
            data: WData::JSON(vault_oracle_redeemer.to_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        .tx_in_script(&vault_oracle_script.script_cbor)
        // vault oracle output
        .tx_out(
            &vault_oracle_script_utxo.output.address,
            &vault_oracle_script_utxo.output.amount,
        )
        .tx_out_inline_datum_value(&WData::JSON(vault_oracle_datum.to_string())); // JSON string datum

    // add intent utxos
    for intent_utxo in intent_utxos {
        tx_builder
            .tx_in(
                &intent_utxo.script_utxo.input.tx_hash,
                intent_utxo.script_utxo.input.output_index,
                &intent_utxo.script_utxo.output.amount,
                &intent_utxo.script_utxo.output.address,
            )
            .tx_in_redeemer_value(&WRedeemer {
                data: WData::JSON(intent_utxo.redeemer.to_string()),
                ex_units: Budget { mem: 0, steps: 0 },
            })
            .tx_in_script(&intent_utxo.script.script_cbor);
    }

    // add intent outputs
    for intent_output in intent_outputs {
        tx_builder.tx_out(&intent_output.address, &intent_output.output_amount);
    }

    tx_builder
        .change_address(my_address)
        .tx_in_collateral(
            &collateral.input.tx_hash,
            collateral.input.output_index,
            &collateral.output.amount,
            &collateral.output.address,
        )
        .select_utxos_from(inputs, 5000000)
        .complete(None)
        .await?;

    Ok(tx_builder.tx_hex())
}
#[cfg(test)]
mod tests {
    use crate::{
        handler::sign_transaction::check_signature_sign_tx, utils::wallet::get_operator_wallet,
    };

    use super::*;
    use dotenv::dotenv;
    use std::env::var;
    use whisky::csl::BaseAddress;
    use whisky::csl::Credential;

    #[tokio::test]
    async fn test_app_sign_tx() {
        dotenv().ok();
        let provider = BlockfrostProvider::new(
            var("BLOCKFROST_PREPROD_PROJECT_ID").unwrap().as_str(),
            "preprod",
        );
        let mut app_owner_wallet = get_operator_wallet();

        let address = BaseAddress::new(
            0,
            &Credential::from_keyhash(
                &app_owner_wallet
                    .payment_account(0, 0)
                    .get_account()
                    .unwrap()
                    .public_key
                    .hash(),
            ),
            &Credential::from_keyhash(
                &app_owner_wallet
                    .payment_account(0, 0)
                    .get_account()
                    .unwrap()
                    .public_key
                    .hash(),
            ),
        )
        .to_address()
        .to_bech32(None)
        .unwrap()
        .to_string();
        println!("result: {:?}", address);

        let utxos = provider.fetch_address_utxos("addr_test1qz675ad696kf4zzt5lz8zy9t0720nspsvcmwfhcp7vufyruyevqwkea4n9wxr2ftrcqk77x6drq5slzpq4ded0kpkwvq89gd6e", None).await.unwrap();
        // let tx_hex = mint_oracle(&"addr_test1qqgetxt6xhz08u9s68km9scj8gjcjlvczrs9ghu4p3s6u8cc0f73w6hkrjxhqhsarjq750fzj4cdv86xjrnr3fw6ljnqwsw386", "addr_test1qz675ad696kf4zzt5lz8zy9t0720nspsvcmwfhcp7vufyruyevqwkea4n9wxr2ftrcqk77x6drq5slzpq4ded0kpkwvq89gd6e", &utxos).unwrap();
        // println!("result: {:?}", tx_hex);

        // let signed_tx = check_signature_sign_tx(&app_owner_wallet, &tx_hex).unwrap();
        // assert!(!signed_tx.is_empty());

        // let result = provider.submit_tx(&signed_tx).await;
        // assert!(
        //     result.is_ok(),
        //     "Transaction submission failed: {:?}",
        //     result.err()
        // );
    }
}
