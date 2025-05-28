use anchor_client::{Client, Cluster};
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::rc::Rc;
use std::str::FromStr;
use escrow;

pub async fn call_create_escrow(
    proposal_id: u64,
    job_id: u64,
    freelancer_pubkey: &str,
    amount: u64,
) -> Result<(), Box<dyn std::error::Error>> {

    // Load client keypair from environment (secure storage)
    // let keypair_path = env::var("SOLANA_KEYPAIR_PATH")
    // .map_err(|_| "SOLANA_KEYPAIR_PATH not set")?;
    // let keypair = Keypair::from_base58_string(
    //     &std::fs::read_to_string(keypair_path)
    //         .map_err(|e| format!("Failed to read keypair: {}", e))?,
    // );

    let keypair = Keypair::new();
    
    let client = Client::new(Cluster::Devnet, Rc::new(keypair));
    let payer = client.payer(); // get the payer keypair

    // Load program with actual program ID
    // TODO: Replace with deployed program ID after `anchor deploy`
    let program_id = Pubkey::from_str("Escrow111111111111111111111111111111111111")?;
    let program = client.program(program_id)?;
    
    


    let (escrow_pda, _bump) = Pubkey::find_program_address(
        &[b"escrow", &proposal_id.to_le_bytes()],
        &program.id(),
    );

    program
        .request()
        .accounts(escrow::accounts::CreateEscrow {
            escrow: escrow_pda,
            client: payer.pubkey(),
            freelancer: Pubkey::from_str(freelancer_pubkey)?,
            authority: payer.pubkey(), // TODO: Set authority to a designated key
            system_program: anchor_lang::solana_program::system_program::ID,
        })
        .args(escrow::instruction::CreateEscrow { proposal_id, job_id, amount})
        .signer(&payer)
        .send()?;

    Ok(())
}