use whisky::{data::PlutusDataJson, *};

use crate::scripts::{
    swap_oracle::{swap_oracle_mint_blueprint, swap_oracle_spend_blueprint, SwapOracleDatum},
    MintPolarity,
};

pub async fn setup_swap_oracle(
    my_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    one_shot_utxo: &UTxO,
    vault_script_hash: &str,
    operator_key: &str,
    swap_charge: i128,
    dd_key: &str,
) -> Result<String, WError> {
    let swap_oracle_mint_blueprint = swap_oracle_mint_blueprint(
        &one_shot_utxo.input.tx_hash,
        one_shot_utxo.input.output_index as i128,
    )?;
    let swap_oracle_spend_blueprint =
        swap_oracle_spend_blueprint(&swap_oracle_mint_blueprint.hash)?;

    let vault_oracle_datum = SwapOracleDatum::setup_swap_oracle_datum(
        vault_script_hash,
        &swap_oracle_mint_blueprint.hash,
        operator_key,
        swap_charge,
        dd_key,
    )?;

    let vault_oracle_output_amount =
        vec![Asset::new_from_str(&swap_oracle_mint_blueprint.hash, "1")];

    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .tx_in(
            &one_shot_utxo.input.tx_hash,
            one_shot_utxo.input.output_index,
            &one_shot_utxo.output.amount,
            &one_shot_utxo.output.address,
        )
        .mint_plutus_script_v3()
        .mint(1, &swap_oracle_mint_blueprint.hash, "")
        .minting_script(&swap_oracle_mint_blueprint.cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(MintPolarity::RMint.to_json_string()),
            ex_units: Budget::default(),
        })
        .tx_out(
            &swap_oracle_spend_blueprint.address,
            &vault_oracle_output_amount,
        )
        .tx_out_inline_datum_value(&WData::JSON(vault_oracle_datum.to_json_string()))
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
    use super::*;
    use crate::utils::wallet::get_operator_wallet;
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
                rt.block_on(test_setup_swap_oracle_tx());
            })
            .unwrap();

        handle.join().unwrap();
    }

    async fn test_setup_swap_oracle_tx() {
        dotenv().ok();
        let kupo_provider = KupoProvider::new(var("KUPO_URL").unwrap().as_str());
        let ogmios_provider = OgmiosProvider::new(var("OGMIOS_URL").unwrap().as_str());
        let app_owner_wallet = get_operator_wallet()
            .with_fetcher(kupo_provider.clone())
            .with_submitter(ogmios_provider.clone());

        let app_operator_key = app_owner_wallet
            .addresses
            .base_address
            .as_ref()
            .unwrap()
            .payment_cred()
            .to_hex();

        let address = app_owner_wallet
            .get_change_address(AddressType::Payment)
            .unwrap()
            .to_string();
        println!("address: {:?}", address);
        let utxos = app_owner_wallet.get_utxos(None, None).await.unwrap();
        let one_shot = utxos[0].clone();
        println!("one_shot: {:?}", one_shot);
        let wallet_utxos = utxos[1..].to_vec();
        let collateral = app_owner_wallet.get_collateral(None).await.unwrap()[0].clone();

        let tx_hex = setup_swap_oracle(
            &address,
            &wallet_utxos,
            &collateral,
            &one_shot,
            "",
            &app_operator_key,
            50,
            &app_operator_key,
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
