use whisky::*;

pub async fn register_cert(
    stake_key_hash: &str,
    my_address: &str,
    inputs: &[UTxO],
) -> Result<String, WError> {
    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .register_stake_certificate(stake_key_hash)
        .change_address(my_address)
        .select_utxos_from(inputs, 5000000)
        .complete_sync(None)?;

    Ok(tx_builder.tx_hex())
}

#[cfg(test)]
mod tests {
    use crate::{
        scripts::swap_intent::swap_intent_withdraw_blueprint, utils::wallet::get_operator_wallet,
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
                rt.block_on(test_app_register_cert());
            })
            .unwrap();

        handle.join().unwrap();
    }
    async fn test_app_register_cert() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let swap_oracle_nft = var("SWAP_ORACLE_NFT").unwrap();
        let swap_oracle_nft_blueprint = swap_intent_withdraw_blueprint(&swap_oracle_nft).unwrap();

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();
        println!("address: {:?}", address);

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();

        let tx_hex = register_cert(&swap_oracle_nft_blueprint.address, &address, &utxos)
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
