use std::env::var;

pub struct AppConfig {
    pub network_id: String,
    pub operator_mnemonic: String,
    pub operator_vkey: String,
    pub blockfrost_api_key: String,
}

impl AppConfig {
    pub fn new() -> AppConfig {
        AppConfig {
            network_id: var("NETWORK_ID").unwrap_or("0".to_string()),
            operator_mnemonic: convert_mnemonic_comma_to_space(
                &var("OPERATOR_SEED_PHRASE").unwrap(),
            ),
            operator_vkey: var("OPERATOR_VKEY").unwrap_or("".to_string()),
            blockfrost_api_key: var("BLOCKFROST_PREPROD_PROJECT_ID").unwrap_or("".to_string()),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

fn convert_mnemonic_comma_to_space(mnemonic: &str) -> String {
    mnemonic.replace(',', " ")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_mnemonic_comma_conversion() {
        let mnemonic = "solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution,solution";
        let expected_mnemonic = "solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution solution";
        let converted = super::convert_mnemonic_comma_to_space(mnemonic);
        assert_eq!(converted, expected_mnemonic);
    }
}
