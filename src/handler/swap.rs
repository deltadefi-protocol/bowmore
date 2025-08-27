use whisky::{data::PlutusDataJson, *};

use crate::scripts::swap_intent::{swap_intent_spend_blueprint, IntentRedeemer, SwapIntentDatum};

pub async fn swap(
    swap_oracle_nft: &str,
    swap_from_asset: &[Asset],
    swap_to_asset: &[Asset],
    user_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    mint_ref_utxo: &UTxO,
) -> Result<String, WError> {
    let swap_intent_blueprint = swap_intent_spend_blueprint(swap_oracle_nft).unwrap();

    let swap_intent_datum = SwapIntentDatum::new(swap_from_asset, swap_to_asset, user_address);

    let mut swap_intent_output_amount = swap_from_asset
        .iter()
        .map(|a| a.clone())
        .collect::<Vec<_>>();
    swap_intent_output_amount.push(Asset::new_from_str(&swap_intent_blueprint.hash, "1"));

    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .mint_plutus_script_v3()
        .mint(1, &swap_intent_blueprint.hash, "")
        .mint_tx_in_reference(
            mint_ref_utxo.input.tx_hash.as_str(),
            mint_ref_utxo.input.output_index,
            &swap_intent_blueprint.hash,
            swap_intent_blueprint.cbor.len() / 2,
        ) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(IntentRedeemer::MintIntent.to_json_string()),
            ex_units: Budget::default(),
        })
        .tx_out(&swap_intent_blueprint.address, &swap_intent_output_amount)
        .tx_out_inline_datum_value(&WData::JSON(swap_intent_datum.to_json_string()))
        .change_address(user_address)
        .tx_in_collateral(
            &collateral.input.tx_hash,
            collateral.input.output_index,
            &collateral.output.amount,
            &collateral.output.address,
        )
        .select_utxos_from(inputs, 5000000)
        .input_for_evaluation(mint_ref_utxo)
        .complete(None)
        .await?;

    Ok(tx_builder.tx_hex())
}

#[cfg(test)]
mod tests {
    use crate::{constant::tx_script, utils::wallet::get_operator_wallet};

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
                rt.block_on(test_swap());
            })
            .unwrap();

        handle.join().unwrap();
    }
    async fn test_swap() {
        dotenv().ok();

        let swap_oracle_nft = var("SWAP_ORACLE_NFT").unwrap();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let mint_ref_utxo = &kupo_provider
            .fetch_utxos(
                tx_script::swap_intent::TX_HASH,
                Some(tx_script::swap_intent::OUTPUT_INDEX),
            )
            .await
            .unwrap()[0];

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();
        println!("address: {:?}", address);

        let from_asset = vec![Asset::new("lovelace".to_string(), "3000000".to_string())];
        let to_asset = vec![Asset::new("lovelace".to_string(), "3000000".to_string())];

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();
        let collateral = app_owner_wallet.get_collateral(None).await.unwrap()[0].clone();

        let tx_hex = swap(
            &swap_oracle_nft,
            &from_asset,
            &to_asset,
            &address,
            &utxos,
            &collateral,
            mint_ref_utxo,
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
