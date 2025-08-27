use whisky::ConstrEnum;
pub mod app_deposit_request;
pub mod deposit_intent;
pub mod lp_token;
pub mod plutus_loader;
pub mod swap_intent;
pub mod swap_oracle;
pub mod types;
pub mod vault;
pub mod vault_oracle;
pub mod withdrawal_intent;
#[derive(Debug, Clone, ConstrEnum)]
pub enum MintPolarity {
    RMint,
    RBurn,
}
