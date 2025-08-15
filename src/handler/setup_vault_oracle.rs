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
    );
    let vault_oracle_spend_blueprint =
        vault_oracle_spend_blueprint(&vault_oracle_mint_blueprint.hash);

    let vault_oracle_datum = VaultOracleDatum::setup_vault_oracle_datum(
        &vault_oracle_mint_blueprint.hash,
        lp_decimal,
        pluggable_logic,
        operator_charge,
        operator_key,
    );

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
