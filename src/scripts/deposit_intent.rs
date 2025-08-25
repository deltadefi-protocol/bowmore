use whisky::{
    data::{byte_string, integer, Address, ByteString, Int, List, Map, OutputReference, Value},
    Asset, ConstrEnum, WError,
};

#[derive(Debug, Clone, ConstrEnum)]
pub enum IntentRedeemer {
    MintIntent,
    BurnIntent(List<Int>, ByteString, List<ByteString>),
}

#[derive(Debug, Clone, ConstrEnum)]
pub enum SignedMessage {
    Message(Int, Map<(ByteString, ByteString), Int>, OutputReference),
}

impl SignedMessage {
    pub fn from_plutus_data(message: &str) -> Result<Self, WError> {
        let plutus_data = whisky::csl::PlutusData::from_hex(message).map_err(|_e| {
            WError::new(
                "Failed to decode message as Plutus data",
                "InvalidMessageError",
            )
        })?;

        let datum_json = whisky::csl::decode_plutus_datum_to_json_value(
            &plutus_data,
            whisky::csl::PlutusDatumSchema::DetailedSchema,
        )
        .map_err(|_err| {
            WError::new("Failed to decode Plutus datum to JSON", "InvalidDatumError")
        })?;

        let fields = datum_json["fields"]
            .as_array()
            .ok_or_else(|| WError::new("Invalid SignedMessage structure", "InvalidDataError"))?;

        let vault_balance_int = fields[0]["int"]
            .as_i64()
            .ok_or_else(|| WError::new("Missing vault balance message", "InvalidDataError"))?;
        let vault_balance = Int::new(vault_balance_int as i128);

        let prices_map_json = &fields[1]["map"];
        let prices_array = prices_map_json
            .as_array()
            .ok_or_else(|| WError::new("Invalid prices map structure", "InvalidDataError"))?;

        let mut prices_map = Map::new(&[]);
        for entry in prices_array {
            let policy_id_byte = entry["k"]["list"][0]["bytes"].as_str().ok_or_else(|| {
                WError::new("Invalid policy ID in prices map", "InvalidDataError")
            })?;
            let asset_name_byte = entry["k"]["list"][1]["bytes"].as_str().ok_or_else(|| {
                WError::new("Invalid asset name in prices map", "InvalidDataError")
            })?;
            let value_int = entry["v"]["int"].as_i64().ok_or_else(|| {
                WError::new("Invalid price value in prices map", "InvalidDataError")
            })?;

            let key = (
                ByteString::new(policy_id_byte),
                ByteString::new(asset_name_byte),
            );
            let value = Int::new(value_int as i128);

            prices_map.insert(key, value);
        }

        let tx_hash = fields[2]["fields"][0]["bytes"]
            .as_str()
            .ok_or_else(|| WError::new("Missing tx hash in UTXO reference", "InvalidDataError"))?;

        let output_index = fields[2]["fields"][1]["int"].as_i64().ok_or_else(|| {
            WError::new("Missing output index in UTXO reference", "InvalidDataError")
        })?;

        let tx_hash_bytes = ByteString::new(tx_hash);
        let output_index_int = Int::new(output_index as i128);

        let utxo_ref_data = Box::new((tx_hash_bytes, output_index_int));
        let utxo_ref = OutputReference::new(utxo_ref_data);

        Ok(SignedMessage::Message(vault_balance, prices_map, utxo_ref))
    }
}

#[derive(Debug, Clone, ConstrEnum)]
pub enum DepositIntentDatum {
    Datum(Address, Value),
}

impl DepositIntentDatum {
    pub fn new(assets: &[Asset], address: &str) -> Self {
        let m_value = Value::from_asset_vec(assets);
        let w_address = whisky::deserialize_address(address);

        let (payment_key_hash, is_script_payment_key) = if w_address.pub_key_hash.is_empty() {
            (w_address.script_hash, true)
        } else {
            (w_address.pub_key_hash, false)
        };

        let (stake_key_hash, is_script_stake_key) = if w_address.stake_key_hash.is_empty() {
            (w_address.stake_key_script_hash, true)
        } else {
            (w_address.stake_key_hash, false)
        };

        let address_datum = Address::new(
            &payment_key_hash,
            Some(&stake_key_hash),
            is_script_payment_key,
            is_script_stake_key,
        );

        DepositIntentDatum::Datum(address_datum, m_value)
    }
}

use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::config::AppConfig;
use crate::scripts::plutus_loader::get_compiled_code_by_index;

pub fn deposit_intent_mint_blueprint(
    oracle_nft: &str,
    lp_decimal: i128,
) -> Result<MintingBlueprint, whisky::WError> {
    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    let compiled_code = get_compiled_code_by_index(1)?; // Using index 1 for deposit intent mint
    blueprint
        .param_script(
            &compiled_code,
            &[
                &byte_string(oracle_nft).to_string(),
                &integer(lp_decimal).to_string(),
            ],
            BuilderDataType::JSON,
        )
        .unwrap();
    Ok(blueprint)
}

pub fn deposit_intent_spend_blueprint(
    oracle_nft: &str,
    lp_decimal: i128,
) -> Result<SpendingBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    let compiled_code = get_compiled_code_by_index(0)?; // Using index 0 for deposit intent spend
    blueprint
        .param_script(
            &compiled_code,
            &[
                &byte_string(oracle_nft).to_string(),
                &integer(lp_decimal).to_string(),
            ],
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
    fn test_deposit_intent_mint_blueprint() {
        dotenv().ok();

        let blueprint = deposit_intent_mint_blueprint(
            "c9e99dda2af8e97d8ccde3254fc1c16926fbbf6508929dad6518d1b83f389e92",
            1000000,
        )
        .unwrap();
        println!("blueprint: {:?}", blueprint);
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_deposit_intent_spend_blueprint() {
        dotenv().ok();

        let blueprint = deposit_intent_spend_blueprint("todo", 1000000).unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
