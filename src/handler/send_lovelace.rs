use whisky::*;

pub async fn send_lovelace(
    address: &str,
    my_address: &str,
    inputs: &[UTxO],
) -> Result<String, WError> {
    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .tx_out(address, &[Asset::new_from_str("lovelace", "3000000")])
        .change_address(my_address)
        .select_utxos_from(inputs, 5000000)
        .complete_sync(None)?;

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
    async fn test_app_sign_tx() {
        dotenv().ok();
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

        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();

        let tx_hex = send_lovelace(
            "addr_test1wzczetkqy2a4sdekc84e9vxsnfyfzzle7u4j2s58qlpnrrcu4cr2q",
            &address,
            &utxos,
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
