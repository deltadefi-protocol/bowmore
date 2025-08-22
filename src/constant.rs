pub mod preprod {
    pub mod unit {
        pub const USDM: &str = "c69b981db7a65e339a6d783755f85a2e03afa1cece9714c55fe4c9135553444d";
        pub const LOVELACE: &str = "lovelace";
    }
}

pub mod mainnet {
    pub mod unit {
        pub const USDM: &str = "c69b981db7a65e339a6d783755f85a2e03afa1cece9714c55fe4c9135553444d"; // TODO: Update mainnet USDM
        pub const LOVELACE: &str = "lovelace";
    }
}

pub mod tx_script {
    pub const ADDRESS: &str = "addr_test1qzn9zp4r0u9j8upcf5vmwyp92rktxkguy82gqjsax5v3x9tpjch2tctwrlw8x5777gukav57r8jaezgmmhq0hp9areuqgpaw9k";
    pub mod vault {
        pub const TX_HASH: &str = "some_tx_hash";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod vault_oracle {
        pub const TX_HASH: &str = "some_tx_hash";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod deposit_intent {
        pub const TX_HASH: &str = "some_tx_hash";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod withdrawal_intent {
        pub const TX_HASH: &str = "some_tx_hash";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod lp_token {
        pub const TX_HASH: &str = "some_tx_hash";
        pub const TX_INDEX: u32 = 0;
    }
}
