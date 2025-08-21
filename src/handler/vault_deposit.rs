use whisky::{data::PlutusDataJson, *};

use crate::scripts::deposit_intent::{
    deposit_intent_spend_blueprint, DepositIntentDatum, IntentRedeemer,
};

pub async fn vault_deposit(
    oracle_nft: &str,
    deposit_assets: &[Asset],
    user_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    lp_decimal: i128,
) -> Result<String, WError> {
    let deposit_intent_blueprint = deposit_intent_spend_blueprint(oracle_nft, lp_decimal).unwrap();

    let deposit_intent_datum = DepositIntentDatum::new(deposit_assets, user_address);

    let mut deposit_intent_output_amount =
        deposit_assets.iter().map(|a| a.clone()).collect::<Vec<_>>();
    deposit_intent_output_amount.push(Asset::new_from_str(&deposit_intent_blueprint.hash, "1"));

    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .mint_plutus_script_v3()
        .mint(1, &deposit_intent_blueprint.hash, "")
        .minting_script(&deposit_intent_blueprint.cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(IntentRedeemer::MintIntent.to_json_string()),
            ex_units: Budget::default(),
        })
        .tx_out(
            &deposit_intent_blueprint.address,
            &deposit_intent_output_amount,
        )
        .tx_out_inline_datum_value(&WData::JSON(deposit_intent_datum.to_json_string()))
        .change_address(user_address)
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
    use crate::utils::wallet::get_operator_wallet;

    use super::*;
    use dotenv::dotenv;
    use std::env::var;
    use whisky::{kupo::KupoProvider, ogmios::OgmiosProvider};

    #[tokio::test]
    async fn test_vault_deposit() {
        dotenv().ok();

        let oracle_nft = var("ORACLE_NFT").unwrap();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();
        println!("address: {:?}", address);

        let deposit_asset = vec![Asset::new("lovelace".to_string(), "3000000".to_string())];
        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();
        let collateral = app_owner_wallet.get_collateral(None).await.unwrap()[0].clone();

        let tx_hex = vault_deposit(
            &oracle_nft,
            &deposit_asset,
            &address,
            &utxos,
            &collateral,
            1000000,
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
