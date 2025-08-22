use serde_json::{from_str, Value};
use std::{collections::HashMap, u8};
use whisky::{
    data::{Int, List},
    Asset, Output, UTxO, WError,
};

use crate::{
    config::AppConfig, constant::mainnet, constant::preprod, utils::blockfrost::get_utxo_by_address,
};

pub fn parse_plutus_address_obj_to_bech32(plutus_data_address_obj: &str, network_id: u8) -> String {
    let plutus_data_address: Value =
        from_str(plutus_data_address_obj).expect("Invalid json string");
    let plutus_data_key_obj = plutus_data_address.get("fields").unwrap();
    let plutus_data_key_list = plutus_data_key_obj.as_array().unwrap();

    let plutus_data_payment_key_obj = &plutus_data_key_list[0];
    let plutus_data_stake_key_obj = &plutus_data_key_list[1];

    let payment_key_hash = plutus_data_payment_key_obj["fields"][0]["bytes"]
        .as_str()
        .unwrap();

    let csl_payment_credential =
        if plutus_data_payment_key_obj["constructor"].as_u64().unwrap() == 0 {
            whisky::csl::Credential::from_keyhash(
                &whisky::csl::Ed25519KeyHash::from_hex(payment_key_hash).unwrap(),
            )
        } else {
            whisky::csl::Credential::from_scripthash(
                &whisky::csl::ScriptHash::from_hex(payment_key_hash).unwrap(),
            )
        };

    if plutus_data_stake_key_obj["constructor"].as_u64().unwrap() == 0 {
        let stake_key_hash = plutus_data_stake_key_obj["fields"][0]["fields"][0]["fields"][0]
            ["bytes"]
            .as_str()
            .unwrap();
        if plutus_data_stake_key_obj["fields"][0]["fields"][0]["constructor"]
            .as_u64()
            .unwrap()
            == 0
        {
            whisky::csl::BaseAddress::new(
                network_id,
                &csl_payment_credential,
                &whisky::csl::Credential::from_keyhash(
                    &whisky::csl::Ed25519KeyHash::from_hex(stake_key_hash).unwrap(),
                ),
            )
            .to_address()
            .to_bech32(None)
            .unwrap()
        } else {
            whisky::csl::BaseAddress::new(
                network_id,
                &csl_payment_credential,
                &whisky::csl::Credential::from_scripthash(
                    &whisky::csl::ScriptHash::from_hex(stake_key_hash).unwrap(),
                ),
            )
            .to_address()
            .to_bech32(None)
            .unwrap()
        }
    } else {
        whisky::csl::EnterpriseAddress::new(network_id, &csl_payment_credential)
            .to_address()
            .to_bech32(None)
            .unwrap()
    }
}

pub fn convert_value_to_usd(
    assets: &[Asset],
    prices: &HashMap<String, i128>,
) -> Result<i128, WError> {
    let mut total_value = 0;
    for asset in assets {
        if let Some(price) = prices.get(&asset.unit()) {
            total_value += price * asset.quantity_i128();
        }
    }
    Ok(total_value)
}

pub fn cal_operator_fee(
    vault_balance: i128,
    hwm_lp_value: i128,
    operator_charge: i128,
) -> Result<i128, WError> {
    if vault_balance < hwm_lp_value {
        Ok(0)
    } else {
        Ok((vault_balance - hwm_lp_value) * operator_charge / 100)
    }
}

pub fn cal_lp_token_amount(
    usd_value: i128,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
    lp_decimal: i128,
) -> Result<i128, WError> {
    let lp_amount = if total_lp == 0 {
        usd_value * lp_decimal
    } else {
        (usd_value * total_lp) / (vault_balance - operator_fee)
    };
    Ok(lp_amount)
}

pub fn combine_assets(assets1: &[Asset], assets2: &[Asset]) -> Vec<Asset> {
    let mut combined = assets1.to_vec();

    for asset2 in assets2 {
        let unit = asset2.unit();

        // Try to find matching asset in the combined list
        if let Some(pos) = combined.iter().position(|a| a.unit() == unit) {
            // If found, sum the quantities
            let new_quantity = combined[pos].quantity_i128() + asset2.quantity_i128();
            combined[pos] = Asset::new_from_str(&unit, &new_quantity.to_string());
        } else {
            // If not found, append the asset
            combined.push(asset2.clone());
        }
    }

    combined
}

pub fn subtract_assets(assets1: &[Asset], assets2: &[Asset]) -> Vec<Asset> {
    let mut result = Vec::new();

    // First copy all assets from assets1
    for asset1 in assets1 {
        let unit = asset1.unit();

        // If this asset exists in assets2, subtract its quantity
        if let Some(asset2) = assets2.iter().find(|a| a.unit() == unit) {
            let remaining = asset1.quantity_i128() - asset2.quantity_i128();
            if remaining > 0 {
                result.push(Asset::new_from_str(&unit, &remaining.to_string()));
            }
        } else {
            // If not in assets2, keep the original amount
            result.push(asset1.clone());
        }
    }

    result
}

pub fn process_deposit_intent(
    utxo: &UTxO,
    prices: &HashMap<String, i128>,
    lp_token_policy_id: &str,
    lp_decimal: i128,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
) -> Result<(Output, Vec<Asset>, i128, i128), WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    match &utxo.output.plutus_data {
        Some(plutus_data) => {
            let datum_json = whisky::csl::decode_plutus_datum_to_json_value(
                &whisky::csl::PlutusData::from_hex(&plutus_data).map_err(|_e| {
                    WError::new("Failed to decode Plutus data", "InvalidDatumError")
                })?,
                whisky::csl::PlutusDatumSchema::DetailedSchema,
            )
            .map_err(|_err| {
                WError::new("Failed to decode Plutus datum to JSON", "InvalidDatumError")
            })?;

            let address_value = datum_json["fields"][0].to_string();
            let address = parse_plutus_address_obj_to_bech32(
                &address_value,
                network_id.parse::<u8>().unwrap(),
            );

            // Extract assets from the value field
            let assets_array = datum_json["fields"][1]["map"]
                .as_array()
                .ok_or_else(|| WError::new("Missing assets array in JSON", "InvalidDataError"))?;

            let mut assets = Vec::new();
            for asset_pair in assets_array {
                let policy_id = asset_pair["k"]["bytes"]
                    .as_str()
                    .ok_or_else(|| WError::new("Missing policy_id in asset", "InvalidDataError"))?;

                let asset = asset_pair["v"]["map"].as_array().ok_or_else(|| {
                    WError::new("Missing asset array in JSON", "InvalidDataError")
                })?;

                for asset_entry in asset {
                    let asset_name = asset_entry["k"]["bytes"].as_str().ok_or_else(|| {
                        WError::new("Missing asset_name in asset", "InvalidDataError")
                    })?;
                    let quantity = asset_entry["v"]["int"].as_i64().ok_or_else(|| {
                        WError::new("Missing quantity in asset", "InvalidDataError")
                    })?;

                    let unit = if asset_name.is_empty() {
                        if policy_id.is_empty() {
                            preprod::unit::LOVELACE.to_string()
                        } else {
                            policy_id.to_string()
                        }
                    } else {
                        format!("{}{}", policy_id, asset_name)
                    };
                    assets.push(Asset::new_from_str(&unit, &quantity.to_string()));
                }
            }

            let usd_value = convert_value_to_usd(&assets, prices)?;

            let lp_amount =
                cal_lp_token_amount(usd_value, vault_balance, total_lp, operator_fee, lp_decimal)?;

            let output = Output {
                address: address.to_string(),
                amount: vec![Asset::new_from_str(
                    lp_token_policy_id,
                    &lp_amount.to_string(),
                )],
                datum: None,
                reference_script: None,
            };

            Ok((output, assets, usd_value, lp_amount))
        }
        None => Err(WError::new(
            "UTxO does not contain Plutus data for DepositIntentDatum",
            "InvalidDatumError",
        )),
    }
}

/// Process a list of deposit intent UTxOs and return the sum of values
///
/// For each UTxO in the input list:
/// - Decodes the Plutus datum
/// - Extracts assets and address
/// - Calculates USD value based on provided prices
/// - Calculates LP token amount
/// - Accumulates total USD value and total LP amount
///
/// UTxOs that fail to process are skipped and their errors are collected
pub fn process_deposit_intents(
    utxos: &[UTxO],
    prices: &HashMap<String, i128>,
    lp_token_policy_id: &str,
    lp_decimal: i128,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
) -> Result<(Vec<Output>, Vec<Asset>, i128, i128, List<Int>), WError> {
    let mut outputs = Vec::new();
    let mut all_assets = Vec::new();
    let mut total_usd_value: i128 = 0;
    let mut total_lp_amount: i128 = 0;
    let mut indices = Vec::new();
    let mut index: i128 = 2;

    for utxo in utxos {
        let (output, assets, usd_value, lp_amount) = process_deposit_intent(
            utxo,
            prices,
            lp_token_policy_id,
            lp_decimal,
            vault_balance,
            total_lp,
            operator_fee,
        )?;

        indices.push(Int::new(index));
        index += 1;
        outputs.push(output);
        all_assets = combine_assets(&all_assets, &assets);
        total_usd_value += usd_value;
        total_lp_amount += lp_amount;
    }

    Ok((
        outputs,
        all_assets,
        total_usd_value,
        total_lp_amount,
        List::new(&indices),
    ))
}

pub fn convert_lp_to_usd(
    lp_amount: i128,
    total_lp: i128,
    vault_balance: i128,
    operator_fee: i128,
) -> Result<i128, WError> {
    let usd_value = (lp_amount * (vault_balance - operator_fee)) / total_lp;
    Ok(usd_value)
}

pub fn cal_lovelace_amount(
    prices: &HashMap<String, i128>,
    usd_value: i128,
) -> Result<i128, WError> {
    let lovelace_amount = usd_value * prices.get(preprod::unit::LOVELACE).unwrap();
    Ok(lovelace_amount)
}

pub fn process_withdrawal_intent(
    utxo: &UTxO,
    prices: &HashMap<String, i128>,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
    ratio: i128, // Ratio of USDM in &
) -> Result<(Output, Vec<Asset>, i128, i128), WError> {
    let AppConfig { network_id, .. } = AppConfig::new();

    let (lovelace_unit, usdm_unit) = if network_id.parse::<i128>().unwrap() == 0 {
        (preprod::unit::LOVELACE, preprod::unit::USDM)
    } else {
        (mainnet::unit::LOVELACE, mainnet::unit::USDM)
    };
    match &utxo.output.plutus_data {
        Some(plutus_data) => {
            let datum_json = whisky::csl::decode_plutus_datum_to_json_value(
                &whisky::csl::PlutusData::from_hex(&plutus_data).map_err(|_e| {
                    WError::new("Failed to decode Plutus data", "InvalidDatumError")
                })?,
                whisky::csl::PlutusDatumSchema::DetailedSchema,
            )
            .map_err(|_err| {
                WError::new("Failed to decode Plutus datum to JSON", "InvalidDatumError")
            })?;

            let address_value = datum_json["fields"][0].to_string();
            let address = parse_plutus_address_obj_to_bech32(
                &address_value,
                network_id.parse::<u8>().unwrap(),
            );

            // Extract lp_amount from the value field
            let lp_amount = datum_json["fields"][1]["int"]
                .as_i64()
                .ok_or_else(|| WError::new("Missing lp amount in JSON", "InvalidDataError"))?;

            let usd_value =
                convert_lp_to_usd(lp_amount.into(), total_lp, vault_balance, operator_fee)?;

            let output = Output {
                address: address.to_string(),
                amount: vec![
                    Asset::new_from_str(usdm_unit, &(usd_value * ratio / 100).to_string()),
                    Asset::new_from_str(
                        lovelace_unit,
                        &cal_lovelace_amount(prices, usd_value).unwrap().to_string(),
                    ),
                ],
                datum: None,
                reference_script: None,
            };

            Ok((
                output,
                vec![
                    Asset::new_from_str(usdm_unit, &(usd_value * ratio / 100).to_string()),
                    Asset::new_from_str(
                        lovelace_unit,
                        &cal_lovelace_amount(prices, usd_value).unwrap().to_string(),
                    ),
                ],
                usd_value,
                lp_amount.into(),
            ))
        }
        None => Err(WError::new(
            "UTxO does not contain Plutus data for WithdrawalIntentDatum",
            "InvalidDatumError",
        )),
    }
}

pub fn process_withdrawal_intents(
    utxos: &[UTxO],
    prices: &HashMap<String, i128>,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
    ratio: i128,
) -> Result<(Vec<Output>, Vec<Asset>, i128, i128, List<Int>), WError> {
    let mut outputs = Vec::new();
    let mut all_assets = Vec::new();
    let mut total_usd_value: i128 = 0;
    let mut total_lp_amount: i128 = 0;
    let mut indices = Vec::new();
    let mut index: i128 = 2;

    for utxo in utxos {
        let (output, assets, usd_value, lp_amount) =
            process_withdrawal_intent(utxo, prices, vault_balance, total_lp, operator_fee, ratio)?;

        indices.push(Int::new(index));
        index += 1;
        outputs.push(output);
        all_assets = combine_assets(&all_assets, &assets);
        total_usd_value += usd_value;
        total_lp_amount += lp_amount;
    }

    Ok((
        outputs,
        all_assets,
        total_usd_value,
        total_lp_amount,
        List::new(&indices),
    ))
}

pub async fn get_utxos_for_withdrawal(
    vault_address: &str,
    withdrawal_amount: &[Asset],
) -> Result<(Vec<UTxO>, Vec<Asset>), WError> {
    let mut selected_utxos = Vec::new();
    let mut unselected = get_utxo_by_address(vault_address).await?;
    let mut selected_assets = Vec::new();

    // Process non-lovelace assets first
    let non_lovelace: Vec<&Asset> = withdrawal_amount
        .iter()
        .filter(|asset| asset.unit() != "lovelace")
        .collect();

    for withdrawal_asset in non_lovelace {
        let target_amount = withdrawal_asset.quantity().parse::<i128>().unwrap();
        let mut collected_amount = 0i128;
        let mut asset_utxos = Vec::new();
        let mut new_unselected_utxos = Vec::new();

        // Split UTxOs into those containing the asset and those that don't
        for utxo in unselected.iter() {
            if utxo
                .output
                .amount
                .iter()
                .any(|a| a.unit() == withdrawal_asset.unit())
            {
                asset_utxos.push(utxo.clone());
            } else {
                new_unselected_utxos.push(utxo.clone());
            }
        }

        // Select UTxOs until we have enough of the asset
        for (i, utxo) in asset_utxos.iter().enumerate() {
            selected_utxos.push(utxo.clone());
            selected_assets = combine_assets(&selected_assets, &utxo.output.amount);

            // Update collected amount
            collected_amount += utxo
                .output
                .amount
                .iter()
                .filter(|a| a.unit() == withdrawal_asset.unit())
                .map(|a| a.quantity().parse::<i128>().unwrap())
                .sum::<i128>();

            if collected_amount >= target_amount {
                // If we have enough, add remaining UTxOs back to unselected
                new_unselected_utxos.extend(asset_utxos.iter().skip(i + 1).cloned());
                break;
            }
        }
        unselected = new_unselected_utxos;
    }

    // Process lovelace
    if let Some(lovelace) = withdrawal_amount
        .iter()
        .find(|asset| asset.unit() == "lovelace")
    {
        let target_amount = lovelace.quantity().parse::<i128>().unwrap() + 2_000_000; // Add min UTxO value
        let mut collected_amount = selected_assets
            .iter()
            .find(|asset| asset.unit() == "lovelace")
            .map_or(0i128, |a| a.quantity().parse::<i128>().unwrap());

        // Select UTxOs until we have enough lovelace
        for utxo in unselected {
            selected_utxos.push(utxo.clone());
            selected_assets = combine_assets(&selected_assets, &utxo.output.amount);

            // Update collected amount
            collected_amount += utxo
                .output
                .amount
                .iter()
                .filter(|a| a.unit() == "lovelace")
                .map(|a| a.quantity().parse::<i128>().unwrap())
                .sum::<i128>();

            if collected_amount >= target_amount {
                break;
            }
        }
    }

    // Calculate return value by subtracting withdrawal amounts from selected assets
    let mut withdrawal_with_min_ada = withdrawal_amount.to_vec();
    if let Some(lovelace) = withdrawal_amount.iter().find(|a| a.unit() == "lovelace") {
        let min_ada_quantity = lovelace.quantity().parse::<i128>().unwrap() + 2_000_000;
        withdrawal_with_min_ada = withdrawal_amount
            .iter()
            .map(|a| {
                if a.unit() == "lovelace" {
                    Asset::new_from_str("lovelace", &min_ada_quantity.to_string())
                } else {
                    a.clone()
                }
            })
            .collect();
    }

    let return_value = subtract_assets(&selected_assets, &withdrawal_with_min_ada);

    Ok((selected_utxos, return_value))
}
