use whisky::*;

use crate::config::AppConfig;

pub async fn get_utxo(tx_hash: &str, tx_index: u32) -> Result<UTxO, WError> {
    let AppConfig {
        network_id,
        blockfrost_api_key,
        ..
    } = AppConfig::new();
    let provider = BlockfrostProvider::new(&blockfrost_api_key, &network_id);
    let utxos = provider
        .fetch_utxos(tx_hash, Some(tx_index))
        .await
        .map_err(|_e| {
            WError::new(
                &format!(
                    "Failed to fetch UTxOs for tx_hash: {}, tx_index: {}",
                    tx_hash, tx_index
                ),
                "FetchError",
            )
        })?;

    Ok(utxos.first().cloned().ok_or_else(|| {
        WError::new(
            &format!(
                "No UTxO found for tx_hash: {}, tx_index: {}",
                tx_hash, tx_index
            ),
            "NotFoundError",
        )
    })?)
}

pub async fn get_utxo_by_address(address: &str) -> Result<Vec<UTxO>, WError> {
    let AppConfig {
        network_id,
        blockfrost_api_key,
        ..
    } = AppConfig::new();
    let provider = BlockfrostProvider::new(&blockfrost_api_key, &network_id);
    let utxos = provider
        .fetch_address_utxos(address, None)
        .await
        .map_err(|_e| {
            WError::new(
                &format!("Failed to fetch UTxOs for address: {}", address),
                "FetchError",
            )
        })?;

    Ok(utxos)
}
