use std::collections::HashMap;

use whisky::{
    data::{ByteString, List, PlutusDataJson},
    *,
};

use crate::{
    config::AppConfig,
    constant::{mainnet, preprod, tx_script::vault},
    scripts::{
        deposit_intent::{IntentRedeemer, SignedMessage},
        lp_token::lp_token_mint_blueprint,
        vault::{vault_spend_blueprint, VaultRedeemer},
        vault_oracle::{vault_oracle_spend_blueprint, ProcessRedeemer, VaultOracleDatum},
        withdrawal_intent::withdrawal_intent_mint_blueprint,
    },
    utils::{
        batch_process::{
            cal_lovelace_amount, cal_operator_fee, create_withdrawal_output_amount,
            get_utxos_for_withdrawal, process_withdrawal_intents,
        },
        kupo::get_utxo,
    },
};

pub async fn process_vault_withdrawal(
    oracle_nft: &str,
    message: &str,
    signatures: Vec<&str>,
    ratio: i128,
    app_oracle_utxo: &UTxO,
    intent_utxos: &[UTxO],
    operator_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    intent_ref_utxo: &UTxO,
    lp_mint_ref_utxo: &UTxO,
    vault_oracle_spend_ref_utxo: &UTxO,
    vault_ref_utxo: &UTxO,
) -> Result<String, WError> {
    // Decode the message to extract vault balance, prices, and UTXO reference
    let decoded_message = SignedMessage::from_plutus_data(message)?;
    let (mut vault_balance, prices_map, utxo_ref) = match &decoded_message {
        SignedMessage::Message(balance, prices, ref_utxo) => (
            balance.clone().int,
            prices
                .map
                .iter()
                .map(|(k, v)| {
                    let policy_id = k.clone().0.bytes;
                    let asset_name = k.clone().1.bytes;
                    if policy_id.is_empty() {
                        (
                            format!("{}{}", preprod::unit::LOVELACE, asset_name),
                            v.clone().int,
                        )
                    } else {
                        (format!("{}{}", policy_id, asset_name), v.clone().int)
                    }
                })
                .collect::<HashMap<String, i128>>(),
            ref_utxo,
        ),
    };
    vault_balance = 18000000;
    let vault_oracle_tx_hash = (*utxo_ref.clone().fields).0.bytes;
    let vault_oracle_output_index = (*utxo_ref.clone().fields).1.int;

    // Destructure vault oracle
    let vault_oracle_utxo =
        get_utxo(&vault_oracle_tx_hash, vault_oracle_output_index as u32).await?;
    let vault_oracle_input_datum: VaultOracleDatum =
        VaultOracleDatum::from_plutus_data(vault_oracle_utxo.output.plutus_data.as_ref().unwrap())?;

    let (
        _app_oracle,
        _pluggable_logic,
        _node_pub_keys,
        total_lp,
        hwm_lp_value,
        operator_charge,
        _operator_key,
        vault_cost,
        _vault_script_hash,
        _deposit_intent_script_hash,
        _withdrawal_intent_script_hash,
        _lp_token_script_hash,
    ) = match &vault_oracle_input_datum {
        VaultOracleDatum::Datum(
            app_oracle,
            pluggable_logic,
            node_pub_keys,
            total_lp,
            hwm_lp_value,
            operator_charge,
            operator_key,
            vault_cost,
            vault_script_hash,
            deposit_intent_script_hash,
            withdrawal_intent_script_hash,
            lp_token_script_hash,
        ) => (
            app_oracle.clone(),
            pluggable_logic.clone(),
            node_pub_keys.clone(),
            total_lp.clone(),
            hwm_lp_value.clone(),
            operator_charge.clone(),
            operator_key.clone(),
            vault_cost.clone(),
            vault_script_hash.clone(),
            deposit_intent_script_hash.clone(),
            withdrawal_intent_script_hash.clone(),
            lp_token_script_hash.clone(),
        ),
    };

    let operator_fee = cal_operator_fee(vault_balance, hwm_lp_value.int, operator_charge.int)?;

    // Batch process withdrawal intents
    let (intent_outputs, total_withdrawal_assets, total_usd_value_change, total_lp_minted, indices) =
        process_withdrawal_intents(
            intent_utxos,
            &prices_map,
            vault_balance,
            total_lp.int,
            operator_fee,
            ratio,
        )?;

    // Create the intent redeemer
    let signatures_bytestring: Vec<ByteString> =
        signatures.iter().map(|s| ByteString::new(s)).collect();
    let withdrawl_intent_redeemer = IntentRedeemer::BurnIntent(
        indices,
        ByteString::new(message),
        List::new(&signatures_bytestring),
    );

    // Create the vault oracle datum
    let updated_vault_oracle_datum = VaultOracleDatum::update_vault_oracle_datum(
        &vault_oracle_input_datum,
        total_lp.int - total_lp_minted,
        vault_balance - operator_fee - total_usd_value_change,
        vault_cost.int - total_usd_value_change,
    );

    // Create blueprints
    let withdrawl_intent_blueprint = withdrawal_intent_mint_blueprint(oracle_nft)?;
    let lp_token_mint_blueprint = lp_token_mint_blueprint(oracle_nft)?;
    let vault_oracle_blueprint = vault_oracle_spend_blueprint(oracle_nft)?;
    let vault_spend_blueprint = vault_spend_blueprint(oracle_nft)?;
    println!(
        "vault_spend_blueprint.address: {}",
        vault_spend_blueprint.address
    );
    let (vault_utxos, return_amount) =
        get_utxos_for_withdrawal(&vault_spend_blueprint.address, &total_withdrawal_assets).await?;

    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        // burn intents
        .mint_plutus_script_v3()
        .mint(
            -(intent_utxos.len() as i128),
            &withdrawl_intent_blueprint.hash,
            "",
        )
        .mint_tx_in_reference(
            intent_ref_utxo.input.tx_hash.as_str(),
            intent_ref_utxo.input.output_index,
            &withdrawl_intent_blueprint.hash,
            withdrawl_intent_blueprint.cbor.len() / 2,
        ) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(withdrawl_intent_redeemer.to_json_string()),
            ex_units: Budget::default(),
        })
        //mint lp token
        .mint_plutus_script_v3()
        .mint(-total_lp_minted, &lp_token_mint_blueprint.hash, "")
        .mint_tx_in_reference(
            lp_mint_ref_utxo.input.tx_hash.as_str(),
            lp_mint_ref_utxo.input.output_index,
            &lp_token_mint_blueprint.hash,
            lp_token_mint_blueprint.cbor.len() / 2,
        ) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(ProcessRedeemer::ProcessWithdrawal.to_json_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        // app oracle ref input
        // .read_only_tx_in_reference(
        //     &app_oracle_utxo.input.tx_hash,
        //     app_oracle_utxo.input.output_index,
        //     None,
        // )
        // .tx_in_inline_datum_present()
        // .input_for_evaluation(app_oracle_utxo)
        // vault oracle input
        .spending_plutus_script_v3()
        .tx_in(
            &vault_oracle_utxo.input.tx_hash,
            vault_oracle_utxo.input.output_index,
            &vault_oracle_utxo.output.amount,
            &vault_oracle_utxo.output.address,
        )
        .tx_in_inline_datum_present()
        .tx_in_redeemer_value(&WRedeemer {
            data: WData::JSON(ProcessRedeemer::ProcessWithdrawal.to_json_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        .spending_tx_in_reference(
            vault_oracle_spend_ref_utxo.input.tx_hash.as_str(),
            vault_oracle_spend_ref_utxo.input.output_index,
            &vault_oracle_blueprint.hash,
            vault_oracle_blueprint.cbor.len() / 2,
        )
        .input_for_evaluation(&vault_oracle_utxo)
        // vault oracle output
        .tx_out(
            &vault_oracle_blueprint.address,
            &vault_oracle_utxo.output.amount,
        )
        .tx_out_inline_datum_value(&WData::JSON(updated_vault_oracle_datum.to_json_string())); // JSON string datum

    // operator output
    if operator_fee > 0 {
        let operator_output_amount =
            create_withdrawal_output_amount(&prices_map, operator_fee, ratio)?;
        tx_builder.tx_out(&operator_address, &operator_output_amount);
    }

    // add vault utxos
    for vault_utxo in vault_utxos {
        tx_builder
            .spending_plutus_script_v3()
            .tx_in(
                &vault_utxo.input.tx_hash,
                vault_utxo.input.output_index,
                &vault_utxo.output.amount,
                &vault_utxo.output.address,
            )
            .tx_in_redeemer_value(&WRedeemer {
                data: WData::JSON(VaultRedeemer::WithdrawFund.to_json_string()),
                ex_units: Budget { mem: 0, steps: 0 },
            })
            .spending_tx_in_reference(
                vault_ref_utxo.input.tx_hash.as_str(),
                vault_ref_utxo.input.output_index,
                &vault_spend_blueprint.hash,
                vault_spend_blueprint.cbor.len() / 2,
            ) // For reference scripts
            .tx_in_inline_datum_present()
            .input_for_evaluation(&vault_utxo);
    }

    // add intent utxos
    for intent_utxo in intent_utxos {
        tx_builder
            .tx_in(
                &intent_utxo.input.tx_hash,
                intent_utxo.input.output_index,
                &intent_utxo.output.amount,
                &intent_utxo.output.address,
            )
            .tx_in_redeemer_value(&WRedeemer {
                data: WData::JSON(ByteString::new("").to_json_string()),
                ex_units: Budget { mem: 0, steps: 0 },
            })
            .spending_tx_in_reference(
                intent_ref_utxo.input.tx_hash.as_str(),
                intent_ref_utxo.input.output_index,
                &withdrawl_intent_blueprint.hash,
                withdrawl_intent_blueprint.cbor.len() / 2,
            ) // For reference scripts
            .tx_in_inline_datum_present()
            .input_for_evaluation(intent_utxo);
    }

    // add intent outputs
    for intent_output in intent_outputs {
        tx_builder.tx_out(&intent_output.address, &intent_output.amount);
    }

    // add vault change outputs
    if !return_amount.is_empty() {
        tx_builder.tx_out(&vault_spend_blueprint.address, &return_amount);
    }

    tx_builder
        .change_address(operator_address)
        .tx_in_collateral(
            &collateral.input.tx_hash,
            collateral.input.output_index,
            &collateral.output.amount,
            &collateral.output.address,
        )
        .select_utxos_from(inputs, 5000000)
        .input_for_evaluation(intent_ref_utxo)
        .input_for_evaluation(lp_mint_ref_utxo)
        .input_for_evaluation(vault_oracle_spend_ref_utxo)
        .input_for_evaluation(vault_ref_utxo)
        .complete(None)
        .await?;

    Ok(tx_builder.tx_hex())
}

#[cfg(test)]
mod tests {
    use crate::{
        constant::tx_script, scripts::withdrawal_intent::withdrawal_intent_spend_blueprint,
        utils::wallet::get_operator_wallet,
    };

    use super::*;
    use dotenv::dotenv;
    use std::env::var;
    use whisky::{kupo::KupoProvider, ogmios::OgmiosProvider};

    #[tokio::test]
    async fn test_process_vault_withdrawal() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let app_oracle_nft = var("APP_ORACLE_NFT").unwrap();
        let oracle_nft = var("ORACLE_NFT").unwrap();

        let message = "d8799f00a29f581cc69b981db7a65e339a6d783755f85a2e03afa1cece9714c55fe4c913445553444dff019f4040ff02d8799f5820be960e14b4b0cf4674b955f159aaa735e68859fc3009a364201a4ab186b0706f00ffff";
        let signatures = vec!["", "", "", ""];
        // let app_oracle_utxo = &kupo_provider
        //     .fetch_address_utxos(
        //         "addr_test1wzxjrlgcp4cm7q95luqxq4ss4zjrr9n2usx9kyaafsn7laqjgxmuj",
        //         Some(&app_oracle_nft),
        //     )
        //     .await
        //     .unwrap()[0];

        let withdrawal_intent_blueprint = withdrawal_intent_spend_blueprint(&oracle_nft).unwrap();
        let intent_utxos = kupo_provider
            .fetch_address_utxos(
                &withdrawal_intent_blueprint.address,
                Some(&withdrawal_intent_blueprint.hash),
            )
            .await
            .unwrap();

        let operator_address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();
        println!("address: {:?}", operator_address);

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();
        let collateral = app_owner_wallet.get_collateral(None).await.unwrap()[0].clone();

        let intent_ref_utxo = &kupo_provider
            .fetch_utxos(
                tx_script::withdrawal_intent::TX_HASH,
                Some(tx_script::withdrawal_intent::OUTPUT_INDEX),
            )
            .await
            .unwrap()[0];
        let lp_mint_ref_utxo = &kupo_provider
            .fetch_utxos(
                tx_script::lp_token::TX_HASH,
                Some(tx_script::lp_token::OUTPUT_INDEX),
            )
            .await
            .unwrap()[0];
        let vault_oracle_spend_ref_utxo = &kupo_provider
            .fetch_utxos(
                tx_script::vault_oracle::TX_HASH,
                Some(tx_script::vault_oracle::OUTPUT_INDEX),
            )
            .await
            .unwrap()[0];
        let vault_spend_ref_utxo = &kupo_provider
            .fetch_utxos(
                tx_script::vault::TX_HASH,
                Some(tx_script::vault::OUTPUT_INDEX),
            )
            .await
            .unwrap()[0];

        let tx_hex = process_vault_withdrawal(
            &oracle_nft,
            message,
            signatures,
            0,
            &intent_ref_utxo, // todo: app_oracle_utxo
            &intent_utxos,
            &operator_address,
            &utxos,
            &collateral,
            intent_ref_utxo,
            lp_mint_ref_utxo,
            vault_oracle_spend_ref_utxo,
            vault_spend_ref_utxo,
        )
        .await
        .unwrap();

        let signed_tx = app_owner_wallet.sign_tx(&tx_hex).unwrap();

        assert!(!signed_tx.is_empty());
        println!("signed_tx: {:?}", signed_tx);

        let result = app_owner_wallet.submit_tx(&signed_tx).await;
        print!("result: {:?}", result);
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }
}
