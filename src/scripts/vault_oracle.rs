use dotenv::dotenv;
use std::env::var;
use whisky::data::{byte_string, output_reference, ByteString, Int, List};
use whisky::ConstrEnum;

#[derive(Debug, Clone, ConstrEnum)]
pub enum VaultOracleDatum {
    Datum(
        ByteString,
        ByteString,
        List<ByteString>,
        Int,
        Int,
        Int,
        ByteString,
        Int,
        ByteString,
        ByteString,
        ByteString,
        ByteString,
    ),
}

impl VaultOracleDatum {
    pub fn setup_vault_oracle_datum(
        oracle_nft: &str,
        lp_decimal: i128,
        pluggable_logic: &str,
        operator_charge: i128,
        operator_key: &str,
    ) -> Result<Self, whisky::WError> {
        dotenv().ok();

        let app_oracle = var("APP_ORACLE_NFT").unwrap();
        let node_pub_key_1 = var("NODE_PUB_KEY_1").unwrap();
        let node_pub_key_2 = var("NODE_PUB_KEY_2").unwrap();
        let node_pub_key_3 = var("NODE_PUB_KEY_3").unwrap();
        let node_pub_key_4 = var("NODE_PUB_KEY_4").unwrap();

        let vault_spend_blueprint = vault_spend_blueprint(&oracle_nft)?;
        let deposit_mint_blueprint = deposit_intent_mint_blueprint(&oracle_nft, lp_decimal)?;
        let withdrawal_intent_script_hash = withdrawal_intent_mint_blueprint(&oracle_nft)?;
        let lp_token_mint_blueprint = lp_token_mint_blueprint(&oracle_nft)?;

        Ok(VaultOracleDatum::Datum(
            ByteString::new(&app_oracle),
            ByteString::new(pluggable_logic),
            List::new(&vec![
                ByteString::new(&node_pub_key_1),
                ByteString::new(&node_pub_key_2),
                ByteString::new(&node_pub_key_3),
                ByteString::new(&node_pub_key_4),
            ]),
            Int::new(0),
            Int::new(0),
            Int::new(operator_charge),
            ByteString::new(operator_key),
            Int::new(0),
            ByteString::new(&vault_spend_blueprint.hash),
            ByteString::new(&deposit_mint_blueprint.hash),
            ByteString::new(&withdrawal_intent_script_hash.hash),
            ByteString::new(&lp_token_mint_blueprint.hash),
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
            whisky::WError::new("Invalid VaultOracleDatum structure", "InvalidDataError")
        })?;

        if fields.len() < 12 {
            return Err(whisky::WError::new(
                "Not enough fields in VaultOracleDatum",
                "InvalidDataError",
            ));
        }

        // Extract app_oracle (field 0)
        let app_oracle_bytes = fields[0]["bytes"]
            .as_str()
            .ok_or_else(|| whisky::WError::new("Missing app_oracle field", "InvalidDataError"))?;
        let app_oracle = ByteString::new(app_oracle_bytes);

        // Extract pluggable_logic (field 1)
        let pluggable_logic_bytes = fields[1]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing pluggable_logic field", "InvalidDataError")
        })?;
        let pluggable_logic = ByteString::new(pluggable_logic_bytes);

        // Extract node_pub_key list (field 2)
        let node_pub_keys_json = &fields[2]["list"];
        let node_pub_keys_array = node_pub_keys_json.as_array().ok_or_else(|| {
            whisky::WError::new("Invalid node_pub_key list structure", "InvalidDataError")
        })?;

        let mut node_pub_keys = Vec::new();
        for key_json in node_pub_keys_array {
            let key_bytes = key_json["bytes"].as_str().ok_or_else(|| {
                whisky::WError::new("Invalid node public key format", "InvalidDataError")
            })?;
            node_pub_keys.push(ByteString::new(key_bytes));
        }
        let node_pub_key_list = List::new(&node_pub_keys);

        // Extract total_lp (field 3)
        let total_lp_int = fields[3]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new("Missing or invalid total_lp field", "InvalidDataError")
        })?;
        let total_lp = Int::new(total_lp_int as i128);

        // Extract hwm_lp_value (field 4)
        let hwm_lp_value_int = fields[4]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new("Missing or invalid hwm_lp_value field", "InvalidDataError")
        })?;
        let hwm_lp_value = Int::new(hwm_lp_value_int as i128);

        // Extract operator_charge (field 5)
        let operator_charge_int = fields[5]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new(
                "Missing or invalid operator_charge field",
                "InvalidDataError",
            )
        })?;
        let operator_charge = Int::new(operator_charge_int as i128);

        // Extract operator_key (field 6)
        let operator_key_bytes = fields[6]["bytes"]
            .as_str()
            .ok_or_else(|| whisky::WError::new("Missing operator_key field", "InvalidDataError"))?;
        let operator_key = ByteString::new(operator_key_bytes);

        // Extract vault_cost (field 7)
        let vault_cost_int = fields[7]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new("Missing or invalid vault_cost field", "InvalidDataError")
        })?;
        let vault_cost = Int::new(vault_cost_int as i128);

        // Extract vault_script_hash (field 8)
        let vault_script_hash_bytes = fields[8]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing vault_script_hash field", "InvalidDataError")
        })?;
        let vault_script_hash = ByteString::new(vault_script_hash_bytes);

        // Extract deposit_intent_script_hash (field 9)
        let deposit_intent_script_hash_bytes = fields[9]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new(
                "Missing deposit_intent_script_hash field",
                "InvalidDataError",
            )
        })?;
        let deposit_intent_script_hash = ByteString::new(deposit_intent_script_hash_bytes);

        // Extract withdrawal_intent_script_hash (field 10)
        let withdrawal_intent_script_hash_bytes =
            fields[10]["bytes"].as_str().ok_or_else(|| {
                whisky::WError::new(
                    "Missing withdrawal_intent_script_hash field",
                    "InvalidDataError",
                )
            })?;
        let withdrawal_intent_script_hash = ByteString::new(withdrawal_intent_script_hash_bytes);

        // Extract lp_token_script_hash (field 11)
        let lp_token_script_hash_bytes = fields[11]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing lp_token_script_hash field", "InvalidDataError")
        })?;
        let lp_token_script_hash = ByteString::new(lp_token_script_hash_bytes);

        Ok(VaultOracleDatum::Datum(
            app_oracle,
            pluggable_logic,
            node_pub_key_list,
            total_lp,
            hwm_lp_value,
            operator_charge,
            operator_key,
            vault_cost,
            vault_script_hash,
            deposit_intent_script_hash,
            withdrawal_intent_script_hash,
            lp_token_script_hash,
        ))
    }

    pub fn update_vault_oracle_datum(
        &self,
        new_total_lp: i128,
        new_hwm_lp_value: i128,
        new_cost: i128,
    ) -> Self {
        match self {
            VaultOracleDatum::Datum(
                app_oracle,
                pluggable_logic,
                node_pub_key,
                _total_lp,
                _hwm_lp_value,
                operator_charge,
                operator_key,
                _vault_cost,
                vault_script_hash,
                deposit_intent_script_hash,
                withdrawal_intent_script_hash,
                lp_token_script_hash,
            ) => VaultOracleDatum::Datum(
                app_oracle.clone(),
                pluggable_logic.clone(),
                node_pub_key.clone(),
                Int::new(new_total_lp),
                Int::new(new_hwm_lp_value),
                operator_charge.clone(),
                operator_key.clone(),
                Int::new(new_cost),
                vault_script_hash.clone(),
                deposit_intent_script_hash.clone(),
                withdrawal_intent_script_hash.clone(),
                lp_token_script_hash.clone(),
            ),
        }
    }
}

#[derive(Debug, Clone, ConstrEnum)]
pub enum ProcessRedeemer {
    ProcessDeposit,
    ProcessWithdrawal,
}

use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::{
    config::AppConfig,
    scripts::{
        deposit_intent::deposit_intent_mint_blueprint, lp_token::lp_token_mint_blueprint,
        plutus_loader::get_compiled_code_by_index, vault::vault_spend_blueprint,
        withdrawal_intent::withdrawal_intent_mint_blueprint,
    },
};

pub fn vault_oracle_mint_blueprint(
    tx_hash: &str,
    index: i128,
) -> Result<MintingBlueprint, whisky::WError> {
    let utxo_ref = output_reference(tx_hash, index);
    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    let compiled_code = get_compiled_code_by_index(5)?; // Using index 5 for vault oracle mint

    blueprint
        .param_script(
            &compiled_code,
            &[&utxo_ref.to_string()],
            BuilderDataType::JSON,
        )
        .unwrap();
    Ok(blueprint)
}

pub fn vault_oracle_spend_blueprint(oracle_nft: &str) -> Result<SpendingBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    let compiled_code = get_compiled_code_by_index(10)?; // Using index 10 for vault oracle spend
    blueprint
        .param_script(
            &compiled_code,
            &[&byte_string(oracle_nft).to_string()],
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

        let blueprint = vault_oracle_mint_blueprint("todo", 0).unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_vault_oracle_spend_blueprint() {
        dotenv().ok();

        let blueprint = vault_oracle_spend_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
