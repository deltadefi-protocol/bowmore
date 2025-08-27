use whisky::data::byte_string;

use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::config::AppConfig;
use crate::scripts::plutus_loader::get_compiled_code_by_index;

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
