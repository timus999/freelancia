use ethers::{
    prelude::*,
    utils::{to_checksum},
};
use std::str::FromStr;

pub async fn generate_signature(message: &str, private_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let wallet = LocalWallet::from_str(private_key)?;
    let signature = wallet.sign_message(message).await?;
    Ok(format!("0x{}", signature))
}


pub fn create_eip712_message(nonce: &str, wallet_address: &str) -> String {
    format!(
        "Welcome to Freelancia!\n\nPlease sign this message to authenticate your wallet.\n\nWallet: {}\nNonce: {}",
        wallet_address, nonce
    )
}

pub fn verify_eip712_signature(
    message: &str,
    signature: &str,
    wallet_address: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let signature_bytes = hex::decode(signature.trim_start_matches("0x"))?;
    let signature = Signature::try_from(signature_bytes.as_slice())?;
    let expected_address: Address = wallet_address.parse()?;
    let recovered = signature.recover(message)?; // use the raw message!
    let is_valid = recovered == expected_address;

    if !is_valid {
        eprintln!(
            "Verification failed: recovered = {}, expected = {}",
            to_checksum(&recovered, None),
            to_checksum(&expected_address, None)
        );
    }
    println!("Expected: {}", to_checksum(&expected_address, None));
    println!("Recovered: {}", to_checksum(&recovered, None));


    Ok(is_valid)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_signature() {
        let private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let wallet_address = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
        let nonce = "testnonce";
        let message = create_eip712_message(nonce, wallet_address);
        let signature = generate_signature(&message, private_key).await.unwrap();
        assert!(signature.starts_with("0x"));
        assert_eq!(signature.len(), 132); // 65 bytes in hex + 0x
    }

    #[tokio::test]
    async fn test_verify_signature() {
        let private_key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let wallet = LocalWallet::from_str(private_key).unwrap();
        let wallet_address = format!("{:#x}", wallet.address()); // get actual address from key
    
        let nonce = "testnonce";
        let message = create_eip712_message(nonce, &wallet_address);
        let signature = generate_signature(&message, private_key).await.unwrap();
        let is_valid = verify_eip712_signature(&message, &signature, &wallet_address).unwrap();
        assert!(is_valid, "Signature verification failed");
    }
    
}