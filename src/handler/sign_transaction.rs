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
        let tx_hex = "84ab00d901028182582072526e8753ee2118942afe94dedfe9f27887c463c415b67a4a4c37016a5c4ef8000182a300581d70e08f4fb3d28ea7fcf12f1609ab8262ccdf5e48c553d25884ceba08a401821a001e8480a1581c11e486bc31467b1afbf13676fb3884e0089664098351c48d772cd36ca14001028201d818586ad8799fd8799fd8799f50aaed0c6eefad4451a6687003a6f81c6ed8799f581c04845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66ffd8799f581cde47016def89cec1e8ff349d044802bce9a845009bd84569db69e585ffffffa140a1401a001e8480ff8258390004845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66e0464447c1f51adaefe1ebfb0dd485a349a70479ced1d198cbdf7fe71b0000000129e2bdb6021a0004afca075820bdaa99eb158414dea0a91d6c727e2268574b23efe6e08ab3b841abe8059a030c09a1581c11e486bc31467b1afbf13676fb3884e0089664098351c48d772cd36ca140010b5820c5854f667cc4c0ffdc17133239c69ba20fd3c588f968e4265af1d6eb6316630c0dd901028182582054e96c49c0b7bb5bfdc365a93cdf641341a104ef231d639da735465ea8e2293e000ed9010281581cfa5136e9e9ecbc9071da73eeb6c9a4ff73cbf436105cf8380d1c525c108258390004845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66e0464447c1f51adaefe1ebfb0dd485a349a70479ced1d198cbdf7fe71b0000000127a8feef111a001e848012d9010282825820dc839b35d68cca788ad6800b06273ef9d90cc6def24a23dcc933635057de6d710082582097eef7899b12bc9b826e2080af500e9b6af10dc9d5b268ad83a9f3e8d9b5b6e100a105a182010082d87980821a0007515c1a08d235c5f5d90103a0".to_string();
        let signed_tx = check_signature_sign_tx(&app_owner_wallet, &tx_hex).unwrap();
        println!("Signed Transaction: {}", signed_tx);
        assert!(!signed_tx.is_empty());
    }

    #[test]
    fn test_sk_sign_tx() {
        dotenv().ok();
        let app_owner_wallet = get_operator_wallet();
        let tx_hex = "84ab00d901028182582072526e8753ee2118942afe94dedfe9f27887c463c415b67a4a4c37016a5c4ef8000182a300581d70e08f4fb3d28ea7fcf12f1609ab8262ccdf5e48c553d25884ceba08a401821a001e8480a1581c11e486bc31467b1afbf13676fb3884e0089664098351c48d772cd36ca14001028201d818586ad8799fd8799fd8799f50881b74cf319647feac191f2085ea2a6ed8799f581c04845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66ffd8799f581cde47016def89cec1e8ff349d044802bce9a845009bd84569db69e585ffffffa140a1401a001e8480ff8258390004845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66e0464447c1f51adaefe1ebfb0dd485a349a70479ced1d198cbdf7fe71b0000000129e2bdb6021a0004afca075820bdaa99eb158414dea0a91d6c727e2268574b23efe6e08ab3b841abe8059a030c09a1581c11e486bc31467b1afbf13676fb3884e0089664098351c48d772cd36ca140010b5820c5854f667cc4c0ffdc17133239c69ba20fd3c588f968e4265af1d6eb6316630c0dd9010281825820901055a9fd822b4a11a0f4ce7a0e9d11a9b2ef21e2800d368a5407fa3cedb8e5000ed9010281581cfa5136e9e9ecbc9071da73eeb6c9a4ff73cbf436105cf8380d1c525c108258390004845038ee499ee8bc0afe56f688f27b2dd76f230d3698a9afcc1b66e0464447c1f51adaefe1ebfb0dd485a349a70479ced1d198cbdf7fe71b0000000127a8feef111a001e848012d9010282825820dc839b35d68cca788ad6800b06273ef9d90cc6def24a23dcc933635057de6d7100825820a3b66ba90e4e7238cdac3771ca8030357fd22562cb970c3a5bb1122a29094cf900a105a182010082d87980821a0007515c1a08d235c5f5d90103a0".to_string();
        let signed_tx = check_signature_sign_tx(&app_owner_wallet, &tx_hex).unwrap();
        println!("Signed Transaction: {}", signed_tx);
        assert!(!signed_tx.is_empty());
    }
}
