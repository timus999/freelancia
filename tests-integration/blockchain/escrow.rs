#[cfg(test)]
mod tests {
    use super::*;
    use anchor_client::{Client, Cluster};
    use solana_program_test::*;
    use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
    use std::str::FromStr;

    async fn setup() -> (ProgramTestContext, Client, Pubkey) {
        let mut program = ProgramTest::default();
        program.add_program("escrow", escrow::id(), None);
        let context = program.start_with_context().await;
        let client = Client::new(Cluster::Localnet, Rc::new(context.payer.insecure_clone()));
        let program_id = escrow::id();
        (context, client, program_id)
    }

    #[tokio::test]
    async fn test_create_escrow() {
        let (mut context, client, program_id) = setup().await;
        let program = client.program(program_id).unwrap();
        let client_keypair = context.payer.insecure_clone();
        let freelancer = Keypair::new();
        let proposal_id = 1;
        let job_id = 1;
        let amount = 1_000_000_000; // 1 SOL

        let (escrow_pda, _bump) = Pubkey::find_program_address(
            &[b"escrow", &proposal_id.to_le_bytes()],
            &program_id,
        );

        let tx = program
            .request()
            .accounts(escrow::accounts::CreateEscrow {
                escrow: escrow_pda,
                client: client_keypair.pubkey(),
                freelancer: freelancer.pubkey(),
                authority: client_keypair.pubkey(),
                system_program: solana_sdk::system_program::id(),
            })
            .args(escrow::instruction::CreateEscrow {
                proposal_id,
                job_id,
                amount,
            })
            .signer(&client_keypair)
            .send()
            .await
            .unwrap();

        let escrow_account: escrow::EscrowAccount = program
            .account(escrow_pda)
            .await
            .unwrap();
        assert_eq!(escrow_account.amount, amount);
        assert_eq!(escrow_account.status, escrow::EscrowStatus::Funded);
    }

    // Add tests for release_payment, raise_dispute, resolve_dispute
}