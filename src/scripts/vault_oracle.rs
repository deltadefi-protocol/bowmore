use whisky::data::{byte_string, output_reference, ByteString, Int, List};
use whisky::ConstrEnum;

#[derive(Debug, Clone, ConstrEnum)]
pub enum VaultOracleDatum {
    Datum(
        ByteString,
        ByteString,
        List<ByteString>,
        Int,
        Int,
        Int,
        ByteString,
        Int,
        ByteString,
        ByteString,
        ByteString,
        ByteString,
    ),
}

impl VaultOracleDatum {
    pub fn setup_vault_oracle_datum(
        oracle_nft: &str,
        lp_decimal: i128,
        pluggable_logic: &str,
        operator_charge: i128,
        operator_key: &str,
    ) -> Self {
        let vault_spend_blueprint = vault_spend_blueprint(&oracle_nft);
        let deposit_mint_blueprint = deposit_intent_mint_blueprint(&oracle_nft, lp_decimal);
        let withdrawal_intent_script_hash = withdrawal_intent_mint_blueprint(&oracle_nft);
        let lp_token_mint_blueprint = lp_token_mint_blueprint(&oracle_nft);

        VaultOracleDatum::Datum(
            ByteString::new("todo: app_oracle"),
            ByteString::new(pluggable_logic),
            List::new(&vec![
                ByteString::new("todo: node_pub_key_1"),
                ByteString::new("todo: node_pub_key_2"),
                ByteString::new("todo: node_pub_key_3"),
                ByteString::new("todo: node_pub_key_4"),
            ]),
            Int::new(0),
            Int::new(0),
            Int::new(operator_charge),
            ByteString::new(operator_key),
            Int::new(0),
            ByteString::new(&vault_spend_blueprint.hash),
            ByteString::new(&deposit_mint_blueprint.hash),
            ByteString::new(&withdrawal_intent_script_hash.hash),
            ByteString::new(&lp_token_mint_blueprint.hash),
        )
    }

    pub fn from_plutus_data(plutus_data_hex: &str) -> Result<Self, whisky::WError> {
        let plutus_data = whisky::csl::PlutusData::from_hex(plutus_data_hex).map_err(|_e| {
            whisky::WError::new(
                "Failed to decode hex string to PlutusData",
                "InvalidDataError",
            )
        })?;

        let datum_json = whisky::csl::decode_plutus_datum_to_json_value(
            &plutus_data,
            whisky::csl::PlutusDatumSchema::DetailedSchema,
        )
        .map_err(|_err| {
            whisky::WError::new("Failed to decode Plutus datum to JSON", "InvalidDatumError")
        })?;

        let fields = datum_json["fields"].as_array().ok_or_else(|| {
            whisky::WError::new("Invalid VaultOracleDatum structure", "InvalidDataError")
        })?;

        if fields.len() < 10 {
            return Err(whisky::WError::new(
                "Not enough fields in VaultOracleDatum",
                "InvalidDataError",
            ));
        }

        // Extract app_oracle (field 0)
        let app_oracle_bytes = fields[0]["bytes"]
            .as_str()
            .ok_or_else(|| whisky::WError::new("Missing app_oracle field", "InvalidDataError"))?;
        let app_oracle = ByteString::new(app_oracle_bytes);

        // Extract pluggable_logic (field 1)
        let pluggable_logic_bytes = fields[1]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing pluggable_logic field", "InvalidDataError")
        })?;
        let pluggable_logic = ByteString::new(pluggable_logic_bytes);

        // Extract node_pub_key list (field 2)
        let node_pub_keys_json = &fields[2]["list"];
        let node_pub_keys_array = node_pub_keys_json.as_array().ok_or_else(|| {
            whisky::WError::new("Invalid node_pub_key list structure", "InvalidDataError")
        })?;

        let mut node_pub_keys = Vec::new();
        for key_json in node_pub_keys_array {
            let key_bytes = key_json["bytes"].as_str().ok_or_else(|| {
                whisky::WError::new("Invalid node public key format", "InvalidDataError")
            })?;
            node_pub_keys.push(ByteString::new(key_bytes));
        }
        let node_pub_key_list = List::new(&node_pub_keys);

        // Extract total_lp (field 3)
        let total_lp_int = fields[3]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new("Missing or invalid total_lp field", "InvalidDataError")
        })?;
        let total_lp = Int::new(total_lp_int as i128);

        // Extract hwm_lp_value (field 4)
        let hwm_lp_value_int = fields[4]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new("Missing or invalid hwm_lp_value field", "InvalidDataError")
        })?;
        let hwm_lp_value = Int::new(hwm_lp_value_int as i128);

        // Extract operator_charge (field 5)
        let operator_charge_int = fields[5]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new(
                "Missing or invalid operator_charge field",
                "InvalidDataError",
            )
        })?;
        let operator_charge = Int::new(operator_charge_int as i128);

        // Extract operator_key (field 6)
        let operator_key_bytes = fields[6]["bytes"]
            .as_str()
            .ok_or_else(|| whisky::WError::new("Missing operator_key field", "InvalidDataError"))?;
        let operator_key = ByteString::new(operator_key_bytes);

        // Extract vault_cost (field 7)
        let vault_cost_int = fields[7]["int"].as_i64().ok_or_else(|| {
            whisky::WError::new("Missing or invalid vault_cost field", "InvalidDataError")
        })?;
        let vault_cost = Int::new(vault_cost_int as i128);

        // Extract vault_script_hash (field 8)
        let vault_script_hash_bytes = fields[8]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing vault_script_hash field", "InvalidDataError")
        })?;
        let vault_script_hash = ByteString::new(vault_script_hash_bytes);

        // Extract deposit_intent_script_hash (field 9)
        let deposit_intent_script_hash_bytes = fields[9]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new(
                "Missing deposit_intent_script_hash field",
                "InvalidDataError",
            )
        })?;
        let deposit_intent_script_hash = ByteString::new(deposit_intent_script_hash_bytes);

        // Extract withdrawal_intent_script_hash (field 10)
        let withdrawal_intent_script_hash_bytes =
            fields[10]["bytes"].as_str().ok_or_else(|| {
                whisky::WError::new(
                    "Missing withdrawal_intent_script_hash field",
                    "InvalidDataError",
                )
            })?;
        let withdrawal_intent_script_hash = ByteString::new(withdrawal_intent_script_hash_bytes);

        // Extract lp_token_script_hash (field 11)
        let lp_token_script_hash_bytes = fields[11]["bytes"].as_str().ok_or_else(|| {
            whisky::WError::new("Missing lp_token_script_hash field", "InvalidDataError")
        })?;
        let lp_token_script_hash = ByteString::new(lp_token_script_hash_bytes);

        Ok(VaultOracleDatum::Datum(
            app_oracle,
            pluggable_logic,
            node_pub_key_list,
            total_lp,
            hwm_lp_value,
            operator_charge,
            operator_key,
            vault_cost,
            vault_script_hash,
            deposit_intent_script_hash,
            withdrawal_intent_script_hash,
            lp_token_script_hash,
        ))
    }

    pub fn update_vault_oracle_datum(
        &self,
        new_total_lp: i128,
        new_hwm_lp_value: i128,
        new_cost: i128,
    ) -> Self {
        match self {
            VaultOracleDatum::Datum(
                app_oracle,
                pluggable_logic,
                node_pub_key,
                _total_lp,
                _hwm_lp_value,
                operator_charge,
                operator_key,
                _vault_cost,
                vault_script_hash,
                deposit_intent_script_hash,
                withdrawal_intent_script_hash,
                lp_token_script_hash,
            ) => VaultOracleDatum::Datum(
                app_oracle.clone(),
                pluggable_logic.clone(),
                node_pub_key.clone(),
                Int::new(new_total_lp),
                Int::new(new_hwm_lp_value),
                operator_charge.clone(),
                operator_key.clone(),
                Int::new(new_cost),
                vault_script_hash.clone(),
                deposit_intent_script_hash.clone(),
                withdrawal_intent_script_hash.clone(),
                lp_token_script_hash.clone(),
            ),
        }
    }
}
// #[derive(Debug, Clone)]
// pub struct VaultOracleDatum(
//     Constr0<
//         Box<(
//             ByteString,
//             ByteString,
//             List<ByteString>,
//             Int,
//             Int,
//             Int,
//             ByteString,
//             Int,
//             ByteString,
//             ByteString,
//             // ByteString, todo: + number of fileds in whisky contructor
//             // ByteString,
//         )>,
//     >,
// );
// impl_constr_wrapper_type!(VaultOracleDatum, 0, [
//   (app_oracle: ByteString, &str),
//   (pluggable_logic: ByteString, &str),
//   (node_pub_key: List<ByteString>, &[ByteString]),
//   (total_lp: Int, i128),
//   (hwm_lp_value: Int, i128),
//   (operator_charge: Int, i128),
//   (operator_key: ByteString, &str),
//   (vault_cost: Int, i128),
//   (vault_script_hash: ByteString, &str),
//   (deposit_intent_script_hash: ByteString, &str),
// //   (withdrawal_intent_script_hash: ByteString, &str),
// //   (lp_token: ByteString, &str),
// ]);

#[derive(Debug, Clone, ConstrEnum)]
pub enum ProcessRedeemer {
    ProcessDeposit,
    ProcessWithdrawal,
}

use whisky::{
    utils::blueprint::{MintingBlueprint, SpendingBlueprint},
    BuilderDataType, LanguageVersion,
};

use crate::{
    config::AppConfig,
    scripts::{
        deposit_intent::deposit_intent_mint_blueprint, lp_token::lp_token_mint_blueprint,
        vault::vault_spend_blueprint, withdrawal_intent::withdrawal_intent_mint_blueprint,
    },
};

pub fn vault_oracle_mint_blueprint(tx_hash: &str, index: i128) -> MintingBlueprint {
    let utxo_ref = output_reference(tx_hash, index);
    let mut blueprint = MintingBlueprint::new(LanguageVersion::V3);
    blueprint
    .param_script(
        "590178010100229800aba2aba1aba0aab9faab9eaab9dab9a488888896600264653001300800198041804800cdc3a400130080024888966002600460106ea800e266446644b300130060018acc004c034dd5004400a2c80722b300130030018acc004c034dd5004400a2c80722c805900b0992cc004c040006264b30013006300c375401115980099198008009bac3011300e375400c44b30010018a508acc004cdd7980918079baa30120010158a518998010011809800a01a4041130030018a50402d13370e0029000a016375a6018601e00316403464b30013002300b375400314bd6f7b63044dd5980798061baa001402864660020026eacc03cc040c040c040c040c030dd5002112cc0040062980103d87a8000899192cc004cdc8803000c56600266e3c018006266e95200033011300f0024bd7045300103d87a80004035133004004301300340346eb8c034004c04000500e18051baa006375c601860126ea800cdc3a400516401c300800130033754011149a26cac8009",
        &[utxo_ref.as_str().unwrap()],
        BuilderDataType::JSON,
    )
    .unwrap();
    blueprint
}

pub fn vault_oracle_spend_blueprint(oracle_nft: &str) -> SpendingBlueprint {
    let AppConfig { network_id, .. } = AppConfig::new();

    let mut blueprint =
        SpendingBlueprint::new(LanguageVersion::V3, network_id.parse().unwrap(), None);
    blueprint
    .param_script(
        "59040f010100229800aba2aba1aba0aab9faab9eaab9dab9a9bae0024888888896600264653001300900198049805000cdc3a400530090024888966002600460126ea800e266453001300f0029807980800146600260186ea800e44646600200200644660060026004005374a90002444646464b30013007301137540031323322598009805000c566002602a6ea800e0051640591598009806800c4c8c8c8cc8966002603c00713300d3758603a00a44b3001002899807802112cc00400a01713230023023003375c604200480fa264600460420066eb4c07c00901d45901b1bac301b001375c603600460360026034002602a6ea800e2c80990130acc004c02cc048dd5000c528c528202230123754002602a60246ea80062c8080c8cc004004dd5980a980b180b180b180b180b180b180b180b180b003112cc0040062980103d87a80008992cc004cdd78021809800c4c018cc058c0500052f5c113300300330180024048602c00280a166002600a601e6ea802a2600466024602600297ae089801198091809980a000a5eb8100e18099809980998099809980998099809980998079baa323259800980318081baa0018992cc004c01cc044dd5000c4c8c8c8c8c8c8c8c8c8c8c8ca60026eb8c0840066eb8c0840326eb8c08402e6eb4c0840266eb4c0840226eb4c08401e6eb8c08401a6eb4c0840166eb8c0840126eb8c08400e6eb8c0840092222222222259800981680644cc070dd6181600a912cc00400a203513230023030003375c605c00481622c815060420026040002603e002603c002603a00260380026036002603400260320026030002602e00260246ea80062c8080c8c9660026010003168acc004c02c0062d1301630133754004808901118089baa00130143015301530113754600460226ea8c050c044dd5000c5900f19198008009bac30140052259800800c5300103d87a80008992cc004c02cc966002601860266ea80062900044dd6980b980a1baa001404864b3001300c3013375400314c103d87a8000899198008009bab30183015375400444b30010018a6103d87a8000899192cc004cdc8a45000018acc004cdc7a44100001898051980d180c00125eb82298103d87a80004059133004004301c00340586eb8c058004c064005017202432330010013756600a60286ea8c014c050dd5001112cc004006298103d87a8000899192cc004cdc880a000c56600266e3c0500062601266032602e00497ae08a60103d87a80004055133004004301b00340546eb8c054004c06000501644c014cc0540052f5c113300300330170024044602a00280988c04cc050005300b375400c91112cc004c0140062b3001301037540150028b20228acc004c0200062b3001301037540150028b20228b201c4038300a37540066e1d20008b2010180480098021baa0098a4d1365640081",
        &[byte_string(oracle_nft).as_str().unwrap()],
        BuilderDataType::JSON,
    )
    .unwrap();
    blueprint
}

#[cfg(test)]
mod tests {

    use super::*;
    use dotenv::dotenv;

    #[test]
    fn test_vault_oracle_mint_blueprint() {
        dotenv().ok();

        let blueprint = vault_oracle_mint_blueprint("todo", 0);
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }

    #[test]
    fn test_vault_oracle_spend_blueprint() {
        dotenv().ok();

        let blueprint = vault_oracle_spend_blueprint("todo");
        assert_eq!(blueprint.hash, "TODO");
        assert_eq!(blueprint.cbor, "TODO");
    }
}
