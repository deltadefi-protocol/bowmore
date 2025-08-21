use whisky::{data::PlutusDataJson, *};

use crate::scripts::{
    vault_oracle::{vault_oracle_mint_blueprint, vault_oracle_spend_blueprint, VaultOracleDatum},
    MintPolarity,
};

pub async fn setup_vault_oracle(
    my_address: &str,
    inputs: &[UTxO],
    collateral: &UTxO,
    one_shot_utxo: &UTxO,
    lp_decimal: i128,
    pluggable_logic: &str,
    operator_charge: i128,
    operator_key: &str,
) -> Result<String, WError> {
    let vault_oracle_mint_blueprint = vault_oracle_mint_blueprint(
        &one_shot_utxo.input.tx_hash,
        one_shot_utxo.input.output_index as i128,
    )?;
    let vault_oracle_spend_blueprint =
        vault_oracle_spend_blueprint(&vault_oracle_mint_blueprint.hash)?;

    let vault_oracle_datum = VaultOracleDatum::setup_vault_oracle_datum(
        &vault_oracle_mint_blueprint.hash,
        lp_decimal,
        pluggable_logic,
        operator_charge,
        operator_key,
    )?;

    let vault_oracle_output_amount =
        vec![Asset::new_from_str(&vault_oracle_mint_blueprint.hash, "1")];

    let mut tx_builder = TxBuilder::new_core();
    tx_builder
        .tx_in(
            &one_shot_utxo.input.tx_hash,
            one_shot_utxo.input.output_index,
            &one_shot_utxo.output.amount,
            &one_shot_utxo.output.address,
        )
        .mint_plutus_script_v3()
        .mint(1, &vault_oracle_mint_blueprint.hash, "")
        .minting_script(&vault_oracle_mint_blueprint.cbor)
        // .mint_tx_in_reference(tx_hash, tx_index, script_hash, script_size) // For reference scripts
        .mint_redeemer_value(&WRedeemer {
            data: WData::JSON(MintPolarity::RMint.to_json_string()),
            ex_units: Budget::default(),
        })
        .tx_out(
            &vault_oracle_spend_blueprint.address,
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
    use crate::utils::wallet::get_operator_wallet;

    use super::*;
    use dotenv::dotenv;
    use std::env::var;
    use whisky::{kupo::KupoProvider, ogmios::OgmiosProvider};

    #[tokio::test]
    async fn test_setup_vault_oracle_tx() {
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

        let tx_hex = setup_vault_oracle(
            &address,
            &wallet_utxos,
            &collateral,
            &one_shot,
            1000000,
            "",
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
