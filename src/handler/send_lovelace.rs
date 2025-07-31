use whisky::*;

pub fn send_lovelace(address: &str, my_address: &str, inputs: &[UTxO]) -> Result<String, WError> {
    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .tx_out(address, &[Asset::new_from_str("lovelace", "5000000")])
        .change_address(my_address)
        .select_utxos_from(inputs, 5000000)
        .complete_sync(None)?;

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
        let tx_hex = send_lovelace(&"addr_test1qqgetxt6xhz08u9s68km9scj8gjcjlvczrs9ghu4p3s6u8cc0f73w6hkrjxhqhsarjq750fzj4cdv86xjrnr3fw6ljnqwsw386", "addr_test1qz675ad696kf4zzt5lz8zy9t0720nspsvcmwfhcp7vufyruyevqwkea4n9wxr2ftrcqk77x6drq5slzpq4ded0kpkwvq89gd6e", &utxos).unwrap();
        println!("result: {:?}", tx_hex);

        let signed_tx = check_signature_sign_tx(&app_owner_wallet, &tx_hex).unwrap();
        assert!(!signed_tx.is_empty());

        let result = provider.submit_tx(&signed_tx).await;
        assert!(
            result.is_ok(),
            "Transaction submission failed: {:?}",
            result.err()
        );
    }
}
