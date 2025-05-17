use ethers::{
    prelude::*,
    utils::{hash_message, to_checksum},
};
use std::str::FromStr;

async fn generate_signature(message: &str, private_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let wallet = LocalWallet::from_str(private_key)?;
    let signature = wallet.sign_message(message).await?;
    Ok(format!("0x{}", signature))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let private_key = "0x59c6995e998f97a5a0044966f09453886bddd4f74efc3b8554c2046df9c6e26d";
    let wallet = LocalWallet::from_str(private_key)?;
    let derived_address = to_checksum(&wallet.address(), None); // 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <nonce> <wallet_address>", args[0]);
        eprintln!("Example: {} testnonce 0x70997970C51812dc3A010C7d01b50e0d17dc79C8", args[0]);
        std::process::exit(1);
    }

    let nonce = args[1].clone();
    let wallet_address = args[2].clone();

    // Validate wallet_address format and match with private key
    if !wallet_address.starts_with("0x") || wallet_address.len() != 42 {
        eprintln!("Error: Invalid wallet_address format. Must be a 42-character hex string starting with '0x'.");
        std::process::exit(1);
    }
    if wallet_address.to_lowercase() != derived_address.to_lowercase() {
        eprintln!("Error: Provided wallet_address {} does not match the address derived from the private key {}", wallet_address, derived_address);
        std::process::exit(1);
    }

    let message = format!(
        "Welcome to Freelancia!\n\nPlease sign this message to authenticate your wallet.\n\nWallet: {}\nNonce: {}",
        wallet_address, nonce
    );
    let signature = generate_signature(&message, private_key).await?;
    println!("Signature: {}", signature);
    Ok(())
}

// wallet generation
// use ethers::prelude::*;
// use ethers::utils::to_checksum;
// use std::str::FromStr;

// fn main() {
//     let private_key = "0x59c6995e998f97a5a0044966f09453886bddd4f74efc3b8554c2046df9c6e26d";
//     println!("Using private key: {}", private_key);

//     let wallet = LocalWallet::from_str(private_key).unwrap();
//     let derived_address = to_checksum(&wallet.address(), None);
//     let expected_address = "0x9b3c42cE61a1A34B38C6a531e985D61cbaA6D19c";

//     println!("Expected: {}", expected_address);
//     println!("Derived:  {}", derived_address);

//     if expected_address.to_lowercase() == derived_address.to_lowercase() {
//         println!("✅ Address matches!");
//     } else {
//         println!("❌ Mismatch!");
//     }
// }
