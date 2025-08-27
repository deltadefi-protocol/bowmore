use whisky::data::{byte_string, output_reference, ByteString, Int};

use whisky::ConstrEnum;
use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::config::AppConfig;
use crate::scripts::plutus_loader::get_compiled_code_by_index;
use crate::scripts::swap_intent::swap_intent_mint_blueprint;

#[derive(Debug, Clone, ConstrEnum)]
pub enum SwapOracleDatum {
    Datum(
        ByteString,
        ByteString,
        ByteString,
        ByteString,
        Int,
        ByteString,
    ),
}

impl SwapOracleDatum {
    pub fn setup_swap_oracle_datum(
        vault_oracle_nft: &str,
        swap_oracle_nft: &str,
        vault_script_hash: &str,
        operator_key: &str,
        swap_charge: i128,
        dd_key: &str,
    ) -> Result<Self, whisky::WError> {
        let vault_spend_blueprint = swap_intent_mint_blueprint(&swap_oracle_nft)?;

        Ok(SwapOracleDatum::Datum(
            ByteString::new(vault_oracle_nft),
            ByteString::new(vault_script_hash),
            ByteString::new(&vault_spend_blueprint.hash),
            ByteString::new(operator_key),
            Int::new(swap_charge),
            ByteString::new(dd_key),
        ))
    }

    pub fn from_plutus_data(plutus_data_hex: &str) -> Result<Self, whisky::WError> {
        let plutus_data = whisky::csl::PlutusData::from_hex(plutus_data_hex).map_err(|_e| {
            whisky::WError::new(
                "Failed to decode hex string to PlutusData",
                "InvalidDataError",
            )
        })?;

        let datum_json = whisky::csl::decode_plutus_datum_to_json_value(
            &plutus_data,
            whisky::csl::PlutusDatumSchema::DetailedSchema,
        )
        .map_err(|_err| {
            whisky::WError::new("Failed to decode Plutus datum to JSON", "InvalidDatumError")
        })?;

        let fields = datum_json["fields"].as_array().ok_or_else(|| {
            whisky::WError::new("Invalid SwapOracleDatum structure", "InvalidDataError")
        })?;

        if fields.len() < 6 {
            return Err(whisky::WError::new(
                "Not enough fields in SwapOracleDatum",
                "InvalidDataError",
            ));
        }

        // Extract vault_oracle_nft (field 0)
        let vault_oracle_nft_bytes = fields[0]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing vault_oracle_nft field", "InvalidDataError")
        })?;
        let vault_oracle_nft = ByteString::new(vault_oracle_nft_bytes);

        // Extract vault_script_hash (field 1)
        let vault_script_hash_bytes = fields[1]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing vault_script_hash field", "InvalidDataError")
        })?;
        let vault_script_hash = ByteString::new(vault_script_hash_bytes);

        // Extract swap_intent_script_hash (field 2)
        let swap_intent_script_hash_bytes = fields[2]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing swap_intent_script_hash field", "InvalidDataError")
        })?;
        let swap_intent_script_hash = ByteString::new(swap_intent_script_hash_bytes);

        // Extract operator_key (field 3)
        let operator_key_bytes = fields[3]["bytes"]
            .as_str()
            .ok_or_else(|| whisky::WError::new("Missing operator_key field", "InvalidDataError"))?;
        let operator_key = ByteString::new(operator_key_bytes);

        // Extract swap_charge (field 4)
        let swap_charge_int = fields[4]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new("Missing or invalid swap_charge field", "InvalidDataError")
        })?;
        let swap_charge = Int::new(swap_charge_int as i128);

        // Extract dd_key (field 5)
        let dd_key_bytes = fields[5]["bytes"]
            .as_str()
            .ok_or_else(|| whisky::WError::new("Missing dd_key field", "InvalidDataError"))?;
        let dd_key = ByteString::new(dd_key_bytes);

        Ok(SwapOracleDatum::Datum(
            vault_oracle_nft,
            vault_script_hash,
            swap_intent_script_hash,
            operator_key,
            swap_charge,
            dd_key,
        ))
    }
}

pub fn swap_oracle_mint_blueprint(
    tx_hash: &str,
    index: i128,
) -> Result<MintingBlueprint, whisky::WError> {
    let utxo_ref = output_reference(tx_hash, index);
    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    let compiled_code = get_compiled_code_by_index(5)?; // Using index 5 for swap oracle mint

    blueprint
        .param_script(
            &compiled_code,
            &[&utxo_ref.to_string()],
            BuilderDataType::JSON,
        )
        .unwrap();
    Ok(blueprint)
}

pub fn swap_oracle_spend_blueprint(
    swap_oracle_nft: &str,
) -> Result<SpendingBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    let compiled_code = get_compiled_code_by_index(17)?; // Using index 17 for swap oracle spend
    blueprint
        .param_script(
            &compiled_code,
            &[&byte_string(swap_oracle_nft).to_string()],
            BuilderDataType::JSON,
        )
        .unwrap();
    Ok(blueprint)
}

#[cfg(test)]
mod tests {

    use super::*;
    use dotenv::dotenv;

    #[test]
    fn test_vault_oracle_mint_blueprint() {
        dotenv().ok();

        let blueprint = swap_oracle_mint_blueprint("todo", 0).unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_vault_oracle_spend_blueprint() {
        dotenv().ok();

        let blueprint = swap_oracle_spend_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
