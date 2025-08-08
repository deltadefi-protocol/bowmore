pub use bowmore_proto::services;
pub mod config;
pub mod handler;
pub mod scripts;
pub mod utils;

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
