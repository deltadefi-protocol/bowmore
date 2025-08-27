use whisky::{
    data::{byte_string, Address, Int},
    ConstrEnum,
};

#[derive(Debug, Clone, ConstrEnum)]
pub enum WithdrawalIntentDatum {
    Datum(Address, Int),
}

impl WithdrawalIntentDatum {
    pub fn new(amount: &str, address: &str) -> Self {
        let w_amount = Int::new(
            amount
                .parse::<i128>()
                .expect("Failed to parse amount as i128"),
        );
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

        WithdrawalIntentDatum::Datum(address_datum, w_amount)
    }
}
use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::{
    config::AppConfig,
    scripts::{lp_token::lp_token_mint_blueprint, plutus_loader::get_compiled_code_by_index},
};

pub fn withdrawal_intent_mint_blueprint(
    oracle_nft: &str,
) -> Result<MintingBlueprint, whisky::WError> {
    let lp_token_mint_blueprint = lp_token_mint_blueprint(oracle_nft)?;
    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    let compiled_code = get_compiled_code_by_index(13)?; // Using index 13 for withdrawal intent mint
    blueprint
        .param_script(
            &compiled_code,
            &[
                &byte_string(oracle_nft).to_string(),
                &byte_string(&lp_token_mint_blueprint.hash).to_string(),
            ],
            BuilderDataType::JSON,
        )
        .unwrap();
    Ok(blueprint)
}

pub fn withdrawal_intent_spend_blueprint(
    oracle_nft: &str,
) -> Result<SpendingBlueprint, whisky::WError> {
    let AppConfig { network_id, .. } = AppConfig::new();
    let lp_token_mint_blueprint = lp_token_mint_blueprint(oracle_nft)?;

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    let compiled_code = get_compiled_code_by_index(12)?; // Using index 12 for withdrawal intent spend
    blueprint
        .param_script(
            &compiled_code,
            &[
                &byte_string(oracle_nft).to_string(),
                &byte_string(&lp_token_mint_blueprint.hash).to_string(),
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
    fn test_withdrawal_intent_mint_blueprint() {
        dotenv().ok();

        let blueprint = withdrawal_intent_mint_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_withdrawal_intent_spend_blueprint() {
        dotenv().ok();

        let blueprint = withdrawal_intent_spend_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
