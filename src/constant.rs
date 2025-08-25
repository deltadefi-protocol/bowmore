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
        pub const TX_HASH: &str =
            "17a0bc5afb4e9d9cc61f92626419d4735cd753a4ec91723cd9a552c49339dadb";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod vault_oracle {
        pub const TX_HASH: &str =
            "1090aaa0e9c33117c6357690e16894a40c9b6c6bac3f83f24f869530a25d6b53";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod deposit_intent {
        pub const TX_HASH: &str =
            "9a37dfdade59323f1152384ca1e636c73d965642a1ca4fc5e5752ea78e713016";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod withdrawal_intent {
        pub const TX_HASH: &str =
            "0e16cf7b6a2c423636ef725a6913aee3cced1f5e315c5b6d197bcd4a80adc164";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod lp_token {
        pub const TX_HASH: &str =
            "1b2a051450d8e8cd2dd773b763ad4ac267620c890977d81f8cea1acfae158b2a";
        pub const OUTPUT_INDEX: u32 = 0;
    }
}
