use whisky::{calculate_tx_hash, CSLParser, WError, Wallet};

use crate::{
    services::{SignTransactionRequest, SignTransactionResponse},
    utils::wallet::get_operator_wallet,
};

pub fn check_signature_sign_tx(wallet: &Wallet, tx_hex: &str) -> Result<String, WError> {
    let signed_tx = wallet.sign_tx(tx_hex).unwrap();

    let mut tx_parser = CSLParser::new();
    let is_transaction_fully_signed =
        tx_parser
            .check_all_required_signers(tx_hex)
            .map_err(WError::from_err(
                "SignTransaction - check_all_required_signers",
            ))?;

    if !is_transaction_fully_signed {
        return Err(WError::new(
            "SignTransaction - check_all_required_signers",
            "Transaction is not fully signed",
        ));
    }
    Ok(signed_tx)
}

pub fn app_sign_tx(tx_hex: &str) -> Result<String, WError> {
    let app_owner_wallet = get_operator_wallet();
    check_signature_sign_tx(&app_owner_wallet, tx_hex)
}

pub fn handler(request: SignTransactionRequest) -> Result<SignTransactionResponse, WError> {
    let app_owner_wallet = get_operator_wallet();
    let tx_hex = request.tx_hex;
    let signed_tx = check_signature_sign_tx(&app_owner_wallet, &tx_hex)?;
    let tx_hash = calculate_tx_hash(&signed_tx)?;
    let reply = SignTransactionResponse { signed_tx, tx_hash };
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    #[test]
    fn test_app_sign_tx() {
        dotenv().ok();
        let app_owner_wallet = get_operator_wallet();
        let tx_hex = "84aa00d901028182582072526e8753ee2118942afe94dedfe9f27887c463c415b67a4a4c37016a5c4ef8000183a300581d70506245b8d10428549499ecfcd0435d5a0b9a3aac2c5bccc824441a7201821a001e8480a1581ceab3a1d125a3bf4cd941a6a0b5d7752af96fae7f5bcc641e8a0b6762a14001028201d818586ad8799fd8799fd8799f508fe62538579144e4b1a438595843973bd8799f581c04845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66ffd8799f581cde47016def89cec1e8ff349d044802bce9a845009bd84569db69e585ffffffa140a1401a3b9aca00ff82581d70ba3efbd72650cbc7d5d7e6bede007cd3cb6730ba1972debf1c2c098f1a3b7c45808258390004845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66e0464447c1f51adaefe1ebfb0dd485a349a70479ced1d198cbdf7fe71aee6815d3021a0003122d075820bdaa99eb158414dea0a91d6c727e2268574b23efe6e08ab3b841abe8059a030c09a1581ceab3a1d125a3bf4cd941a6a0b5d7752af96fae7f5bcc641e8a0b6762a140010b5820b08227937b026d4bbcdff2ebda80397b3d740c1b1d1ab5ec996c2ed60db2117d0dd9010281825820403b20f8168c4b3b0d2efae949b6235d6db801b98509f843d4361a49e840aadf000ed9010281581cfa5136e9e9ecbc9071da73eeb6c9a4ff73cbf436105cf8380d1c525c108258390004845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66e0464447c1f51adaefe1ebfb0dd485a349a70479ced1d198cbdf7fe71a002dc6c0111a001e8480a205a182010082d87980821939af1a0044736807d901028158b558b30101009800aba2a6011e581cfa5136e9e9ecbc9071da73eeb6c9a4ff73cbf436105cf8380d1c525c00a6010746332d6d696e740048c8c8c8c88c88966002646464646464660020026eb0c038c03cc03cc03cc03cc03cc03cc03cc03cc030dd5180718061baa0072259800800c52844c96600266e3cdd71808001005c528c4cc00c00c00500d1808000a01c300c300d002300b001300b002300900130063754003149a26cac8028dd7000ab9a5573caae7d5d09f5d90103a0".to_string();
        let signed_tx = check_signature_sign_tx(&app_owner_wallet, &tx_hex).unwrap();
        println!("Signed Transaction: {}", signed_tx);
        assert!(!signed_tx.is_empty());
    }
}
