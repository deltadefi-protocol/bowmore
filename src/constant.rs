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
            "a035146fda294332981b557cd629e0659351afc9f6992edc1498459fc4e9c3d3";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod vault_oracle {
        pub const TX_HASH: &str =
            "a27df169406a36e904aab8ce600ead5a1b28f3bf770d61ec58cb7dfbc11b259e";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod deposit_intent {
        pub const TX_HASH: &str =
            "d672877e91a477e060846f394dee4e44a77eda95ecc01c7c12a37eafb98d51d3";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod withdrawal_intent {
        pub const TX_HASH: &str =
            "1bf6a4992dd4d5673e268949b94aa603c7dc7261705c6b6047c181b1e4a42ce8";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod lp_token {
        pub const TX_HASH: &str =
            "56fb50f6d62545a13d6eba0cfcca29cc5de11e3601d819795ce8334116204bed";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod swap_intent {
        pub const TX_HASH: &str =
            "65bf338cb594eeb541a15327157ba8d069d770d678d291e0270beb3ed3b3edc8";
        pub const OUTPUT_INDEX: u32 = 0;
    }
    pub mod app_deposit_request {
        pub const TX_HASH: &str =
            "dc839b35d68cca788ad6800b06273ef9d90cc6def24a23dcc933635057de6d71";
        pub const OUTPUT_INDEX: u32 = 0;
    }
}

pub const MIN_UTXO_LOVELACE: u64 = 3_000_000;
