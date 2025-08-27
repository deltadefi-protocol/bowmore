use whisky::data::byte_string;
use whisky::ConstrEnum;

#[derive(Debug, Clone, ConstrEnum)]
pub enum VaultRedeemer {
    WithdrawFund,
    PluggableLogic,
}

use whisky::{
    utils::blueprint::{SpendingBlueprint, WithdrawalBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::{config::AppConfig, scripts::plutus_loader::get_compiled_code_by_index};

pub fn vault_spend_blueprint(oracle_nft: &str) -> Result<SpendingBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    let compiled_code = get_compiled_code_by_index(7)?; // Using index 7 for vault spend
    blueprint
    .param_script(
        &compiled_code,
        &[&byte_string(oracle_nft).to_string()],
        BuilderDataType::JSON,
    )
    .unwrap();
    Ok(blueprint)
}

pub fn vault_withdraw_blueprint(oracle_nft: &str) -> Result<WithdrawalBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint = WithdrawalBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap());
    let compiled_code = get_compiled_code_by_index(8)?; // Using index 8 for vault withdraw
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
    fn test_vault_spend_blueprint() {
        dotenv().ok();

        let blueprint = vault_spend_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_vault_withdraw_blueprint() {
        dotenv().ok();

        let blueprint = vault_withdraw_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
