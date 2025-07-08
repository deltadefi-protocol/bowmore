use crate::config::AppConfig;
use whisky::Wallet;

pub fn get_operator_wallet() -> Wallet {
    let app_config = AppConfig::new();
    let owner_mnemonic = app_config.operator_mnemonic;
    Wallet::new_mnemonic(&owner_mnemonic)
}
