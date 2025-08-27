use whisky::{
    data::byte_string, utils::blueprint::MintingBlueprint, BuilderDataType, LanguageVersion,
};
use crate::scripts::plutus_loader::get_compiled_code_by_index;

pub fn lp_token_mint_blueprint(oracle_nft: &str) -> Result<MintingBlueprint, whisky::WError> {
    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    let compiled_code = get_compiled_code_by_index(3)?; // Using index 3 for lp token mint
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
    fn test_lp_token_mint_blueprint_blueprint() {
        dotenv().ok();

        let blueprint = lp_token_mint_blueprint("todo").unwrap();
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
