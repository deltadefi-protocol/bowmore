use whisky::{
    data::{byte_string, ByteString, Constr0, Credential, Value}, Asset, ConstrEnum
};
use dotenv::dotenv;
use std::env::var;

use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::config::AppConfig;


#[derive(Debug, Clone, ConstrEnum)]
pub enum UserAccount {
    UserSpotAccount(Constr0<Box<(ByteString, Credential, Credential)>>),
    UserFundingAccount(Constr0<Box<(ByteString, Credential, Credential)>>),
    UserMobileAccount(Constr0<Box<(ByteString, Credential, Credential)>>),
}

#[derive(Debug, Clone, ConstrEnum)]
pub enum AppDepositRequestDatum {
    Datum(UserAccount, Value),
}

impl AppDepositRequestDatum {
    pub fn new(assets: &[Asset], account: Constr0<Box<(ByteString, Credential, Credential)>>) -> Self {
        let m_value = Value::from_asset_vec(assets);

        // todo
        AppDepositRequestDatum::Datum(UserAccount::UserSpotAccount(account), m_value)
    }
}

pub fn app_deposit_request_mint_blueprint() -> Result<MintingBlueprint, whisky::WError> {
    dotenv().ok();
    let app_oracle_nft = var("APP_ORACLE_NFT").unwrap();
    let compiled_code = var("APP_DEPOSIT_REQUEST_MINT_CBOR").unwrap();

    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    blueprint
    .param_script(
        &compiled_code,
        &[&byte_string(&app_oracle_nft).to_string()],
        BuilderDataType::JSON,
    )
    .unwrap();
    Ok(blueprint)
}

pub fn app_deposit_request_spend_blueprint() -> Result<SpendingBlueprint, whisky::WError> {
    dotenv().ok();
    let app_oracle_nft = var("APP_ORACLE_NFT").unwrap();
    let compiled_code = var("APP_DEPOSIT_REQUEST_SPEND_CBOR").unwrap();

    let AppConfig { network_id, .. } = AppConfig::new();
    let mut blueprint = SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    blueprint
    .param_script(
        &compiled_code,
        &[&byte_string(&app_oracle_nft).to_string()],
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
    fn test_withdrawal_intent_mint_blueprint() {
        dotenv().ok();

        let blueprint = app_deposit_request_mint_blueprint().unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_withdrawal_intent_spend_blueprint() {
        dotenv().ok();

        let blueprint = app_deposit_request_spend_blueprint().unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
