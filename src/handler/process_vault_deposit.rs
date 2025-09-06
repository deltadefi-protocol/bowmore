use std::collections::HashMap;

use whisky::{
    data::{ByteString, List, PlutusDataJson},
    *,
};

use crate::{
    constant::preprod,
    scripts::{
        app_deposit_request::{
            app_deposit_request_mint_blueprint, app_deposit_request_spend_blueprint,
            AppDepositRequestDatum,
        },
        deposit_intent::{deposit_intent_mint_blueprint, IntentRedeemer, SignedMessage},
        lp_token::lp_token_mint_blueprint,
        vault_oracle::{vault_oracle_spend_blueprint, ProcessRedeemer, VaultOracleDatum},
        MintPolarity,
    },
    utils::{
        batch_process::{cal_operator_fee, process_deposit_intents},
        kupo::get_utxo,
    },
};

pub struct AccountInfo {
    pub account_type: String,
    pub account_id: String,
    pub master_key: (String, bool),
    pub operation_key: (String, bool),
}

pub async fn process_vault_deposit(
    oracle_nft: &str,
    message: &str,
    signatures: Vec<&str>,
    account_info: &AccountInfo,
    app_oracle_utxo: &UTxO,
    intent_utxos: &[UTxO],
    operator_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    lp_decimal: i128,
    intent_ref_utxo: &UTxO,
    lp_mint_ref_utxo: &UTxO,
    vault_oracle_spend_ref_utxo: &UTxO,
    app_deposit_request_mint_ref_utxo: &UTxO,
) -> Result<String, WError> {
    // Decode the message to extract vault balance, prices, and UTXO reference
    let decoded_message = SignedMessage::from_plutus_data(message)?;
    let (vault_balance, prices_map, utxo_ref) = match &decoded_message {
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
            _withdrawal_intent_script_hash,
            _lp_token_script_hash,
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
            _withdrawal_intent_script_hash.clone(),
            _lp_token_script_hash.clone(),
        ),
    };

    let operator_fee = cal_operator_fee(vault_balance, hwm_lp_value.int, operator_charge.int)?;

    // Create blueprints
    let deposit_intent_blueprint = deposit_intent_mint_blueprint(oracle_nft, lp_decimal)?;
    let lp_token_mint_blueprint = lp_token_mint_blueprint(oracle_nft)?;
    let vault_oracle_blueprint = vault_oracle_spend_blueprint(oracle_nft)?;
    let app_deposit_request_spend_blueprint = app_deposit_request_spend_blueprint()?;
    let app_deposit_request_mint_blueprint = app_deposit_request_mint_blueprint()?;

    // Batch process deposit intents
    let (intent_outputs, total_deposit_asset, total_usd_value_change, total_lp_minted, indices) =
        process_deposit_intents(
            intent_utxos,
            &prices_map,
            &lp_token_mint_blueprint.hash,
            lp_decimal,
            vault_balance,
            total_lp.int,
            operator_fee,
        )?;

    let mut app_deposit_request_output_amount = total_deposit_asset.clone();
    app_deposit_request_output_amount.push(Asset::new_from_str(
        &app_deposit_request_mint_blueprint.hash,
        "1",
    ));

    // Create the intent redeemer
    let signatures_bytestring: Vec<ByteString> =
        signatures.iter().map(|s| ByteString::new(s)).collect();
    let deposit_intent_redeemer = IntentRedeemer::BurnIntent(
        indices,
        ByteString::new(message),
        List::new(&signatures_bytestring),
    );

    // Create the vault oracle datum
    let updated_vault_oracle_datum = VaultOracleDatum::update_vault_oracle_datum(
        &vault_oracle_input_datum,
        total_lp.int + total_lp_minted,
        vault_balance - operator_fee + total_usd_value_change,
        vault_cost.int + total_usd_value_change,
    );

    // Create the app deposit request datum
    let app_deposit_request_datum = AppDepositRequestDatum::new(
        &total_deposit_asset,
        &account_info.account_type.as_str(),
        &account_info.account_id.as_str(),
        (
            &account_info.master_key.0.as_str(),
            account_info.master_key.1,
        ),
        (
            &account_info.operation_key.0.as_str(),
            account_info.operation_key.1,
        ),
    );

    let mut vault_oracle_output_amount = vault_oracle_utxo.output.amount.clone();
    vault_oracle_output_amount[0] = Asset::new_from_str("lovelace", "3000000"); // todo: dont hardcode 3000000 as min utxo

    // Build the transaction
    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        // burn intents
        .mint_plutus_script_v3()
        .mint(
            -(intent_utxos.len() as i128),
            &deposit_intent_blueprint.hash,
            "",
        )
        .mint_tx_in_reference(
            intent_ref_utxo.input.tx_hash.as_str(),
            intent_ref_utxo.input.output_index,
            &deposit_intent_blueprint.hash,
            deposit_intent_blueprint.cbor.len() / 2,
        ) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(deposit_intent_redeemer.to_json_string()),
            ex_units: Budget::default(),
        })
        // mint app deposit request
        .mint_plutus_script_v3()
        .mint(1, &app_deposit_request_mint_blueprint.hash, "")
        .mint_tx_in_reference(
            &app_deposit_request_mint_ref_utxo.input.tx_hash,
            app_deposit_request_mint_ref_utxo.input.output_index,
            &app_deposit_request_mint_blueprint.hash,
            &app_deposit_request_mint_blueprint.cbor.len() / 2,
        ) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(MintPolarity::RMint.to_json_string()),
            ex_units: Budget::default(),
        })
        .tx_out(
            &app_deposit_request_spend_blueprint.address,
            &app_deposit_request_output_amount,
        )
        .tx_out_inline_datum_value(&WData::JSON(app_deposit_request_datum.to_json_string()))
        //mint lp token
        .mint_plutus_script_v3()
        .mint(total_lp_minted, &lp_token_mint_blueprint.hash, "")
        .mint_tx_in_reference(
            lp_mint_ref_utxo.input.tx_hash.as_str(),
            lp_mint_ref_utxo.input.output_index,
            &lp_token_mint_blueprint.hash,
            lp_token_mint_blueprint.cbor.len() / 2,
        ) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(ProcessRedeemer::ProcessDeposit.to_json_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
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
            data: WData::JSON(ProcessRedeemer::ProcessDeposit.to_json_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        .spending_tx_in_reference(
            vault_oracle_spend_ref_utxo.input.tx_hash.as_str(),
            vault_oracle_spend_ref_utxo.input.output_index,
            &vault_oracle_blueprint.hash,
            vault_oracle_blueprint.cbor.len() / 2,
        )
        .input_for_evaluation(&vault_oracle_utxo)
        // For reference scripts
        // app oracle ref input
        .read_only_tx_in_reference(
            &app_oracle_utxo.input.tx_hash,
            app_oracle_utxo.input.output_index,
            None,
        )
        .input_for_evaluation(app_oracle_utxo)
        // vault oracle output
        .tx_out(&vault_oracle_blueprint.address, &vault_oracle_output_amount)
        .tx_out_inline_datum_value(&WData::JSON(updated_vault_oracle_datum.to_json_string())); // JSON string datum

    // add intent utxos
    for intent_utxo in intent_utxos {
        println!("intent_utxo: {:?}", intent_utxo);
        tx_builder
            .spending_plutus_script_v3()
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
                &deposit_intent_blueprint.hash,
                deposit_intent_blueprint.cbor.len() / 2,
            ) // For reference scripts
            .tx_in_inline_datum_present()
            .input_for_evaluation(intent_utxo);
    }

    // add intent outputs
    for intent_output in intent_outputs {
        tx_builder.tx_out(&intent_output.address, &intent_output.amount);
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
        .input_for_evaluation(app_deposit_request_mint_ref_utxo)
        .complete(None)
        .await?;

    Ok(tx_builder.tx_hex())
}

#[cfg(test)]
mod tests {
    use crate::{
        constant::tx_script, scripts::deposit_intent::deposit_intent_spend_blueprint,
        utils::wallet::get_operator_wallet,
    };

    use super::*;
    use dotenv::dotenv;
    use std::env::var;
    use whisky::{kupo::KupoProvider, ogmios::OgmiosProvider};

    #[test]
    fn my_async_task() {
        let handle = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(test_process_vault_deposit());
            })
            .unwrap();

        handle.join().unwrap();
    }

    async fn test_process_vault_deposit() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let app_oracle_nft = var("APP_ORACLE_NFT").unwrap();
        let oracle_nft = var("ORACLE_NFT").unwrap();

        let message = "d8799f00a29f581cc69b981db7a65e339a6d783755f85a2e03afa1cece9714c55fe4c913445553444dff019f4040ff02d8799f582035d201e8957797ce3276ab89a8e976b80a20da0dd3d3c49c0b45d62cf68aa31400ffff"; // todo
        let sig_1 = "ed02c6fc132de95940e07e4438ce277022af5b91462400b607b435380a33b36b0f8f3ffdf8bc63edf1a3ced76ce977d1627782fcc2700c3b71a41f99eaf8c903";
        let sig_2 = "6a33119040dfaadaccd506bce42e79ae0bd3d8bbf27c72b1f07b4af8d9e1b753018ededd4ee1e8ad01442c8468e64905a35ee6678d48b110800549419e921d00";
        let sig_3 = "e006028cb0f3b52d3bc7d574b87792582e0f1c9da7ac379779e801f7355053d142514616d4cbb6c60fed706adf4320ef997873e1f8907112ff58d27295d4af05";
        let sig_4 = "9f4af36cef07f87c7b113124f989128058e38c9bdb781cbea81e88efc53f00b4d7f69fff816d368f9206be18218d1cc55730c3e96a2381d4934956d501669801";
        let signatures = vec![sig_1, sig_2, sig_3, sig_4];
        let app_oracle_utxo = &kupo_provider
            .fetch_address_utxos(
                "addr_test1wr3u744257jgnn4n30ttdw7peal8szjnjaskq8xz33v500qwswtux", // todo
                Some(&app_oracle_nft),
            )
            .await
            .unwrap()[0];

        let deposit_intent_blueprint =
            deposit_intent_spend_blueprint(&oracle_nft, 1000000).unwrap();
        let intent_utxos = kupo_provider
            .fetch_address_utxos(
                &deposit_intent_blueprint.address,
                Some(&deposit_intent_blueprint.hash),
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

        let account_info = AccountInfo {
            account_type: "spot_account".to_string(),
            account_id: "062cba14-4f94-42fd-91ac-747a55851660".to_string(),
            master_key: (
                "04845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66".to_string(),
                false,
            ),
            operation_key: (
                "de47016def89cec1e8ff349d044802bce9a845009bd84569db69e585".to_string(),
                false,
            ),
        };

        let intent_ref_utxo = &kupo_provider
            .fetch_utxos(
                tx_script::deposit_intent::TX_HASH,
                Some(tx_script::deposit_intent::OUTPUT_INDEX),
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
        let app_deposit_request_mint_ref_utxo = &kupo_provider
            .fetch_utxos(
                tx_script::app_deposit_request::TX_HASH,
                Some(tx_script::app_deposit_request::OUTPUT_INDEX),
            )
            .await
            .unwrap()[0];

        let tx_hex = process_vault_deposit(
            &oracle_nft,
            message,
            signatures,
            &account_info,
            &app_oracle_utxo,
            &intent_utxos,
            &operator_address,
            &utxos,
            &collateral,
            1000000,
            &intent_ref_utxo,
            &lp_mint_ref_utxo,
            &vault_oracle_spend_ref_utxo,
            &app_deposit_request_mint_ref_utxo,
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
