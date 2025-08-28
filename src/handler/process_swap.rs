use whisky::{
    data::{ByteString, PlutusDataJson},
    *,
};

use crate::{
    scripts::{
        swap_intent::{
            swap_intent_spend_blueprint, swap_intent_withdraw_blueprint, IntentRedeemer,
            SwapIntentWithdrawRedeemer,
        },
        swap_oracle::SwapOracleDatum,
        vault::{vault_spend_blueprint, VaultRedeemer},
    },
    utils::batch_process::{get_utxos_for_withdrawal, process_swap_intents},
};

pub struct AccountInfo {
    pub account_type: String,
    pub account_id: String,
    pub master_key: (String, bool),
    pub operation_key: (String, bool),
}

pub async fn process_swap(
    swap_oracle_nft: &str,
    swap_oracle_utxo: &UTxO,
    intent_utxos: &[UTxO],
    operator_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    intent_ref_utxo: &UTxO,
    vault_ref_utxo: &UTxO,
) -> Result<String, WError> {
    // Batch process swap intents
    let (intent_outputs, withdraw_asset, receive_asset, indices) =
        process_swap_intents(intent_utxos)?;

    let vault_oracle_input_datum: SwapOracleDatum =
        SwapOracleDatum::from_plutus_data(swap_oracle_utxo.output.plutus_data.as_ref().unwrap())?;

    let (
        vault_oracle_nft,
        _vault_script_hash,
        _swap_intent_script_hash,
        operator_key,
        _swap_charge,
        dd_key,
    ) = match &vault_oracle_input_datum {
        SwapOracleDatum::Datum(
            vault_oracle_nft,
            vault_script_hash,
            swap_intent_script_hash,
            operator_key,
            swap_charge,
            dd_key,
        ) => (
            vault_oracle_nft.clone(),
            vault_script_hash.clone(),
            swap_intent_script_hash.clone(),
            operator_key.clone(),
            swap_charge.clone(),
            dd_key.clone(),
        ),
    };

    // Create blueprints
    let swap_intent_blueprint = swap_intent_spend_blueprint(swap_oracle_nft)?;
    let withdraw_blueprint = swap_intent_withdraw_blueprint(swap_oracle_nft)?;
    let vault_spend_blueprint = vault_spend_blueprint(&vault_oracle_nft.bytes)?;

    let (vault_utxos, return_amount) =
        get_utxos_for_withdrawal(&vault_spend_blueprint.address, &withdraw_asset).await?;

    let swap_intent_withdraw_redeemer = SwapIntentWithdrawRedeemer::BurnIntent(indices);

    // Build the transaction
    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        // burn intents
        .mint_plutus_script_v3()
        .mint(
            -(intent_utxos.len() as i128),
            &swap_intent_blueprint.hash,
            "",
        )
        .mint_tx_in_reference(
            intent_ref_utxo.input.tx_hash.as_str(),
            intent_ref_utxo.input.output_index,
            &swap_intent_blueprint.hash,
            swap_intent_blueprint.cbor.len() / 2,
        ) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(IntentRedeemer::BurnIntent.to_json_string()),
            ex_units: Budget::default(),
        })
        // swap oracle ref input
        .read_only_tx_in_reference(
            &swap_oracle_utxo.input.tx_hash,
            swap_oracle_utxo.input.output_index,
            None,
        );
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
                data: WData::JSON(VaultRedeemer::PluggableLogic.to_json_string()),
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
                &swap_intent_blueprint.hash,
                swap_intent_blueprint.cbor.len() / 2,
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
        .tx_out(&vault_spend_blueprint.address, &receive_asset)
        .change_address(operator_address)
        .tx_in_collateral(
            &collateral.input.tx_hash,
            collateral.input.output_index,
            &collateral.output.amount,
            &collateral.output.address,
        )
        .select_utxos_from(inputs, 5000000)
        .required_signer_hash(&operator_key.bytes)
        // .required_signer_hash(&dd_key.bytes)
        .withdrawal_plutus_script_v3()
        .withdrawal(&withdraw_blueprint.address, 0)
        .withdrawal_redeemer_value(&WRedeemer {
            data: WData::JSON(swap_intent_withdraw_redeemer.to_json_string()),
            ex_units: Budget::default(),
        })
        .withdrawal_tx_in_reference(
            intent_ref_utxo.input.tx_hash.as_str(),
            intent_ref_utxo.input.output_index,
            &swap_intent_blueprint.hash,
            swap_intent_blueprint.cbor.len() / 2,
        )
        .input_for_evaluation(intent_ref_utxo)
        .complete(None)
        .await?;

    Ok(tx_builder.tx_hex())
}

#[cfg(test)]
mod tests {
    use crate::{
        constant::tx_script, scripts::swap_oracle::swap_oracle_spend_blueprint,
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
                rt.block_on(test_process_swap());
            })
            .unwrap();

        handle.join().unwrap();
    }

    async fn test_process_swap() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let swap_oracle_nft = var("SWAP_ORACLE_NFT").unwrap();
        let swap_oracle_nft_blueprint = swap_oracle_spend_blueprint(&swap_oracle_nft).unwrap();
        let swap_oracle_utxo = kupo_provider
            .fetch_address_utxos(&swap_oracle_nft_blueprint.address, Some(&swap_oracle_nft))
            .await
            .unwrap()[0]
            .clone();

        let swap_intent_blueprint = swap_intent_spend_blueprint(&swap_oracle_nft).unwrap();
        let intent_utxos = kupo_provider
            .fetch_address_utxos(
                &swap_intent_blueprint.address,
                Some(&swap_intent_blueprint.hash),
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
                tx_script::swap_intent::TX_HASH,
                Some(tx_script::swap_intent::OUTPUT_INDEX),
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

        let tx_hex = process_swap(
            &swap_oracle_nft,
            &swap_oracle_utxo,
            &intent_utxos,
            &operator_address,
            &utxos,
            &collateral,
            &intent_ref_utxo,
            &vault_spend_ref_utxo,
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
