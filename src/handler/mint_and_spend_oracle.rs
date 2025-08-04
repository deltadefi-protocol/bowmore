use whisky::*;

pub struct MintToken {
    pub to_mint_asset: Asset,
    pub redeemer: String,
    pub script: ProvidedScriptSource,
}

pub struct TokenOutput {
    pub output_amount: Vec<Asset>,
    pub datum: String,
    pub script: ProvidedScriptSource,
}

pub async fn one_shot_mint_and_spend(
    to_mint: &MintToken,
    token_output: &TokenOutput,
    my_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    one_shot_utxo: &UTxO,
) -> Result<String, WError> {
    let mut tx_builder = TxBuilder::new_core();
    let MintToken {
        to_mint_asset,
        redeemer,
        script: minting_script,
    } = to_mint;

    let TokenOutput {
        output_amount,
        datum,
        script: spending_script,
    } = token_output;

    let spending_script_hash = get_script_hash(
        &spending_script.script_cbor,
        spending_script.language_version.clone(),
    )?;

    tx_builder
        .tx_in(
            &one_shot_utxo.input.tx_hash,
            one_shot_utxo.input.output_index,
            &one_shot_utxo.output.amount,
            &one_shot_utxo.output.address,
        )
        .mint_plutus_script_v3()
        .mint(
            to_mint_asset.quantity_i128(),
            &to_mint_asset.policy(),
            &to_mint_asset.name(),
        )
        .minting_script(&minting_script.script_cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(redeemer.to_string()),
            ex_units: Budget { mem: 0, steps: 0 },
        })
        .tx_out(&spending_script_hash, output_amount)
        .tx_out_inline_datum_value(&WData::JSON(datum.to_string()))
        .change_address(my_address)
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
