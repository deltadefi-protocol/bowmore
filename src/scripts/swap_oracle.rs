use whisky::data::{byte_string, output_reference};

use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::config::AppConfig;
use crate::scripts::plutus_loader::get_compiled_code_by_index;

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

pub fn swap_oracle_spend_blueprint(oracle_nft: &str) -> Result<SpendingBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    let compiled_code = get_compiled_code_by_index(17)?; // Using index 17 for swap oracle spend
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
