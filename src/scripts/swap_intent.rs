use whisky::data::{byte_string, Address, Int, List, Value};

use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};
use whisky::{Asset, ConstrEnum};

use crate::config::AppConfig;
use crate::scripts::plutus_loader::get_compiled_code_by_index;

#[derive(Debug, Clone, ConstrEnum)]

pub enum SwapIntentWithdrawRedeemer {
    BurnIntent(List<Int>),
}

#[derive(Debug, Clone, ConstrEnum)]
pub enum SwapIntentDatum {
    Datum(Address, Value, Value),
}

impl SwapIntentDatum {
    pub fn new(from_assets: &[Asset], to_assets: &[Asset], address: &str) -> Self {
        let from_value = Value::from_asset_vec(from_assets);
        let to_value = Value::from_asset_vec(to_assets);
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

        SwapIntentDatum::Datum(address_datum, from_value, to_value)
    }
}

pub fn swap_intent_mint_blueprint(oracle_nft: &str) -> Result<MintingBlueprint, whisky::WError> {
    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    let compiled_code = get_compiled_code_by_index(18)?; // Using index 18 for swap intent mint
    blueprint
        .param_script(
            &compiled_code,
            &[&byte_string(oracle_nft).to_string()],
            BuilderDataType::JSON,
        )
        .unwrap();
    Ok(blueprint)
}

pub fn swap_intent_spend_blueprint(oracle_nft: &str) -> Result<SpendingBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    let compiled_code = get_compiled_code_by_index(18)?; // Using index 18 for swap intent spend
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
    fn test_deposit_intent_mint_blueprint() {
        dotenv().ok();

        let blueprint = swap_intent_mint_blueprint(
            "c9e99dda2af8e97d8ccde3254fc1c16926fbbf6508929dad6518d1b83f389e92",
        )
        .unwrap();
        println!("blueprint: {:?}", blueprint);
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_deposit_intent_spend_blueprint() {
        dotenv().ok();

        let blueprint = swap_intent_spend_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
