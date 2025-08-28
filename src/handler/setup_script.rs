use whisky::*;

use crate::constant::tx_script;

pub async fn setup_tx_script(
    my_address: &str,
    inputs: &[UTxO],
    script_cbor: &str,
) -> Result<String, WError> {
    let output_amount = vec![Asset::new_from_str("lovelace", "50000000")];
    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .tx_out(tx_script::ADDRESS, &output_amount)
        .tx_out_reference_script(&script_cbor, Some(LanguageVersion::V3))
        .change_address(my_address)
        .select_utxos_from(inputs, 5000000)
        .complete(None)
        .await?;

    Ok(tx_builder.tx_hex())
}

#[cfg(test)]
mod tests {
    use crate::{
        scripts::{
            deposit_intent::deposit_intent_mint_blueprint, lp_token::lp_token_mint_blueprint,
            swap_intent::swap_intent_spend_blueprint, vault::vault_spend_blueprint,
            vault_oracle::vault_oracle_spend_blueprint,
            withdrawal_intent::withdrawal_intent_mint_blueprint,
        },
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
                rt.block_on(test_tx_out_all_scripts());
            })
            .unwrap();

        handle.join().unwrap();
    }
    async fn test_tx_out_all_scripts() {
        test_tx_out_vault_script().await;
        test_tx_out_vault_oracle_script().await;
        test_tx_out_deposit_intent_script().await;
        test_tx_out_withdrawal_intent_script().await;
        test_tx_out_lp_token_script().await;
        test_tx_out_swap_intent_script().await;
    }

    async fn test_tx_out_vault_script() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());

        let oracle_nft = var("ORACLE_NFT").unwrap();
        let vault_spend_blueprint = vault_spend_blueprint(&oracle_nft).unwrap();

        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();
        let tx_hex = setup_tx_script(&address, &utxos, &vault_spend_blueprint.cbor)
            .await
            .unwrap();

        let signed_tx = app_owner_wallet.sign_tx(&tx_hex).unwrap();
        let result = app_owner_wallet.submit_tx(&signed_tx).await;
        print!("vault result: {:?}", result);
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }

    async fn test_tx_out_vault_oracle_script() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());

        let oracle_nft = var("ORACLE_NFT").unwrap();
        let vault_oracle_blueprint = vault_oracle_spend_blueprint(&oracle_nft).unwrap();

        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();

        let tx_hex = setup_tx_script(&address, &utxos, &vault_oracle_blueprint.cbor)
            .await
            .unwrap();

        let signed_tx = app_owner_wallet.sign_tx(&tx_hex).unwrap();
        let result = app_owner_wallet.submit_tx(&signed_tx).await;
        print!("vault oracle result: {:?}", result);
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }

    async fn test_tx_out_deposit_intent_script() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());

        let oracle_nft = var("ORACLE_NFT").unwrap();
        let deposit_intent_blueprint = deposit_intent_mint_blueprint(&oracle_nft, 1000000).unwrap();

        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();

        let tx_hex = setup_tx_script(&address, &utxos, &deposit_intent_blueprint.cbor)
            .await
            .unwrap();

        let signed_tx = app_owner_wallet.sign_tx(&tx_hex).unwrap();
        let result = app_owner_wallet.submit_tx(&signed_tx).await;
        print!("deposit intent result: {:?}", result);
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }

    async fn test_tx_out_withdrawal_intent_script() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());

        let oracle_nft = var("ORACLE_NFT").unwrap();
        let withdrawal_intent_blueprint = withdrawal_intent_mint_blueprint(&oracle_nft).unwrap();

        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();

        let tx_hex = setup_tx_script(&address, &utxos, &withdrawal_intent_blueprint.cbor)
            .await
            .unwrap();

        let signed_tx = app_owner_wallet.sign_tx(&tx_hex).unwrap();
        let result = app_owner_wallet.submit_tx(&signed_tx).await;
        print!("withdrawal intent result: {:?}", result);
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }

    async fn test_tx_out_lp_token_script() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());

        let oracle_nft = var("ORACLE_NFT").unwrap();
        let lp_token_blueprint = lp_token_mint_blueprint(&oracle_nft).unwrap();

        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();

        let tx_hex = setup_tx_script(&address, &utxos, &lp_token_blueprint.cbor)
            .await
            .unwrap();

        let signed_tx = app_owner_wallet.sign_tx(&tx_hex).unwrap();
        let result = app_owner_wallet.submit_tx(&signed_tx).await;
        print!("lp token result: {:?}", result);
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }

    async fn test_tx_out_swap_intent_script() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());

        let swap_oracle_nft = var("SWAP_ORACLE_NFT").unwrap();
        let swap_intent_blueprint = swap_intent_spend_blueprint(&swap_oracle_nft).unwrap();

        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();

        let tx_hex = setup_tx_script(&address, &utxos, &swap_intent_blueprint.cbor)
            .await
            .unwrap();

        let signed_tx = app_owner_wallet.sign_tx(&tx_hex).unwrap();
        let result = app_owner_wallet.submit_tx(&signed_tx).await;
        print!("swap intent result: {:?}", result);
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }
}
