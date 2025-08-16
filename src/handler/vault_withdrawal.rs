use whisky::{data::PlutusDataJson, *};

use crate::scripts::{
    deposit_intent::IntentRedeemer,
    lp_token::lp_token_mint_blueprint,
    withdrawal_intent::{withdrawal_intent_spend_blueprint, WithdrawalIntentDatum},
};

pub async fn vault_withdrawal(
    oracle_nft: &str,
    withdrawal_amount: &str,
    user_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
) -> Result<String, WError> {
    let lp_token_mint_blueprint = lp_token_mint_blueprint(oracle_nft)?;
    let withdrawal_intent_blueprint = withdrawal_intent_spend_blueprint(oracle_nft)?;

    let withdrawal_intent_datum = WithdrawalIntentDatum::new(withdrawal_amount, user_address);

    let withdrawl_intent_output_amount = vec![
        Asset::new_from_str(&withdrawal_intent_blueprint.hash, "1"),
        Asset::new_from_str(&lp_token_mint_blueprint.hash, withdrawal_amount),
    ];

    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .mint_plutus_script_v3()
        .mint(1, &withdrawal_intent_blueprint.hash, "")
        .minting_script(&withdrawal_intent_blueprint.cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(IntentRedeemer::MintIntent.to_json_string()),
            ex_units: Budget::default(),
        })
        .tx_out(
            &withdrawal_intent_blueprint.address,
            &withdrawl_intent_output_amount,
        )
        .tx_out_inline_datum_value(&WData::JSON(withdrawal_intent_datum.to_json_string()))
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

    #[tokio::test]
    async fn test_vault_withdrawal() {
        dotenv().ok();

        let oracle_nft = var("ORACLE_NFT").unwrap();
        let provider = BlockfrostProvider::new(
            var("BLOCKFROST_PREPROD_PROJECT_ID").unwrap().as_str(),
            "preprod",
        );
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(provider.clone())
            .with_submitter(provider.clone());

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();
        println!("address: {:?}", address);

        
        let withdrawal_amount ="1000000";
        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();
        let collateral = app_owner_wallet.get_collateral(None).await.unwrap()[0].clone();

        let tx_hex = vault_withdrawal(
            &oracle_nft,
            withdrawal_amount,
            &address,
            &utxos,
            &collateral,
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
