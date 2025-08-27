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
            "ced0797b69ae897ae1d35d624597918571fd5edf094636a4b52c1ef7b8fc64d2";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod vault_oracle {
        pub const TX_HASH: &str =
            "6de5334354b351d1313f157d99e5f832417f77e08a488d13f600be196f679f9c";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod deposit_intent {
        pub const TX_HASH: &str =
            "304a68c06506c675ad6ae24ec4e4fededd1cb7d0ea69256144599c48c5c13621";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod withdrawal_intent {
        pub const TX_HASH: &str =
            "ff3db23b7554f39542b38ae2d08fc3bf8852ef0e3cf5c838574da5dd9831c7a5";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod lp_token {
        pub const TX_HASH: &str =
            "846df646b88eb6a1dc409fa5989986ae4e8fd4a2887b420bec84ec2a7230e75b";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod swap_intent {
        pub const TX_HASH: &str = "";
        pub const OUTPUT_INDEX: u32 = 0;
    }
}

pub const MIN_UTXO_LOVELACE: u64 = 3_000_000;
