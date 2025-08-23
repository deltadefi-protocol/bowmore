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
            "f4ee51e7e5a8ba824296fc57fc86dcd40167f99176c2ca2fd64a3e80c631021c";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod vault_oracle {
        pub const TX_HASH: &str =
            "c706a9e976d3564fa7a4fb90d264a192e5e77229480e5a5954eb972c93f14a3c";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod deposit_intent {
        pub const TX_HASH: &str =
            "b4f18061a840dcdbe81bb399b726346b56e138761056a1fc88077063fea419f0";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod withdrawal_intent {
        pub const TX_HASH: &str =
            "cc7eef4d63d9e457266cd7ae556abed9b3395c2802e662c103138e82e5eada30";
        pub const TX_INDEX: u32 = 0;
    }
    pub mod lp_token {
        pub const TX_HASH: &str =
            "d51faee3a81bdf2635034f09e4fe67cc2c660ba2568c5abebba3692a27f4b54c";
        pub const TX_INDEX: u32 = 0;
    }
}
