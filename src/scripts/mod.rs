use whisky::ConstrEnum;

pub mod deposit_intent;
pub mod lp_token;
pub mod types;
pub mod vault;
pub mod vault_oracle;
pub mod withdrawal_intent;
#[derive(Debug, Clone, ConstrEnum)]
pub enum MintPolarity {
    RMint,
    RBurn,
}
