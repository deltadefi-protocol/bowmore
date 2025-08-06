use whisky::{data::PlutusDataJson, *};

use crate::{
    config::AppConfig,
    scripts::deposit_intent::{self, DepositIntentDatum, IntentRedeemer},
};

pub async fn vault_deposit(
    oracle_nft: &str,
    deposit_assets: &[Asset],
    user_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
) -> Result<String, WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let deposit_intent_blueprint =
        deposit_intent::deposit_intent_mint_blueprint(oracle_nft, 1000000);
    let deposit_intent_script_address = whisky::script_to_address(
        network_id.parse().unwrap(),
        &deposit_intent_blueprint.hash,
        None,
    );
    let deposit_intent_datum = DepositIntentDatum::new(deposit_assets, user_address);

    let deposit_intent_output_amount =
        vec![Asset::new_from_str(&deposit_intent_blueprint.hash, "1")];

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
            &deposit_intent_script_address,
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
