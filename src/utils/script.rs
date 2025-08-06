use crate::scripts::{
    deposit_intent::DepositIntentDatum, withdrawal_intent::WithdrawalIntentDatum,
};
use whisky::{
    data::{Address, Int, Value},
    *,
};

pub fn to_deposit_intent_datum(assets: &[Asset], address: &str) -> DepositIntentDatum {
    let m_value = Value::from_asset_vec(assets);
    let w_address = whisky::deserialize_address(address);

    let (payment_key_hash, is_script_payment_key) = if w_address.pub_key_hash.is_empty() {
        (w_address.script_hash, true)
    } else {
        (w_address.pub_key_hash, false)
    };

    let (stake_key_hash, is_script_stake_key) = if w_address.stake_key_hash.is_empty() {
        (w_address.stake_key_script_hash, true)
    } else {
        (w_address.stake_key_hash, false)
    };

    let address_datum = Address::new(
        &payment_key_hash,
        Some(&stake_key_hash),
        is_script_payment_key,
        is_script_stake_key,
    );

    DepositIntentDatum::Datum(address_datum, m_value)
}

pub fn to_withdrawal_intent_datum(amount: &str, address: &str) -> WithdrawalIntentDatum {
    let w_amount = Int::new(
        amount
            .parse::<i128>()
            .expect("Failed to parse amount as i128"),
    );
    let w_address = whisky::deserialize_address(address);

    let (payment_key_hash, is_script_payment_key) = if w_address.pub_key_hash.is_empty() {
        (w_address.script_hash, true)
    } else {
        (w_address.pub_key_hash, false)
    };

    let (stake_key_hash, is_script_stake_key) = if w_address.stake_key_hash.is_empty() {
        (w_address.stake_key_script_hash, true)
    } else {
        (w_address.stake_key_hash, false)
    };

    let address_datum = Address::new(
        &payment_key_hash,
        Some(&stake_key_hash),
        is_script_payment_key,
        is_script_stake_key,
    );

    WithdrawalIntentDatum::Datum(address_datum, w_amount)
}
