use whisky::{Asset, Output, UTxO, WError};

pub fn convert_value_to_usd(assets: &[Asset], prices: &[(String, i128)]) -> Result<i128, WError> {
    let mut total_value = 0;
    for asset in assets {
        if let Some((_, price)) = prices
            .iter()
            .find(|(policy_id, _)| policy_id == &asset.unit())
        {
            total_value += price * asset.quantity_i128();
        }
    }
    Ok(total_value)
}

pub fn cal_lp_token_amount(
    usd_value: i128,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
    lp_decimal: Option<i128>,
) -> Result<i128, WError> {
    match lp_decimal {
        Some(decimal) => {
            let lp_amount = usd_value * decimal;
            Ok(lp_amount)
        }
        None => {
            let lp_amount = (usd_value * total_lp) / (vault_balance - operator_fee);
            Ok(lp_amount)
        }
    }
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

pub fn process_deposit_intent(
    utxo: &UTxO,
    prices: &[(String, i128)],
    lp_token_policy_id: &str,
    lp_decimal: Option<i128>,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
) -> Result<(Output, Vec<Asset>, i128, i128), WError> {
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

            // In a real implementation, you'd extract the correct fields based on your datum structure
            // For now, assuming fields are "constructor", "fields", where fields[0] is address and fields[1] is value
            let address = datum_json["fields"][0]["bytes"]
                .as_str()
                .ok_or_else(|| WError::new("Missing address in JSON", "InvalidDataError"))?;

            // Extract assets from the value field
            let assets_array = datum_json["fields"][1]["map"]
                .as_array()
                .ok_or_else(|| WError::new("Missing assets array in JSON", "InvalidDataError"))?;

            let mut assets = Vec::new();
            for asset_pair in assets_array {
                let policy_id = asset_pair["k"]["bytes"]
                    .as_str()
                    .ok_or_else(|| WError::new("Missing policy_id in asset", "InvalidDataError"))?;
                let quantity = asset_pair["v"]["int"]
                    .as_i64()
                    .ok_or_else(|| WError::new("Missing quantity in asset", "InvalidDataError"))?;

                assets.push(Asset::new_from_str(policy_id, &quantity.to_string()));
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
    prices: &[(String, i128)],
    lp_token_policy_id: &str,
    lp_decimal: Option<i128>,
    vault_balance: i128,
    total_lp: i128,
    operator_fee: i128,
) -> (Vec<Output>, Vec<Asset>, i128, i128, Vec<WError>) {
    let mut outputs = Vec::new();
    let mut all_assets = Vec::new();
    let mut total_usd_value: i128 = 0;
    let mut total_lp_amount: i128 = 0;
    let mut errors = Vec::new();

    for utxo in utxos {
        match process_deposit_intent(
            utxo,
            prices,
            lp_token_policy_id,
            lp_decimal,
            vault_balance,
            total_lp,
            operator_fee,
        ) {
            Ok((output, assets, usd_value, lp_amount)) => {
                outputs.push(output);
                all_assets = combine_assets(&all_assets, &assets);
                total_usd_value += usd_value;
                total_lp_amount += lp_amount;
            }
            Err(error) => errors.push(error),
        }
    }

    (
        outputs,
        all_assets,
        total_usd_value,
        total_lp_amount,
        errors,
    )
}
