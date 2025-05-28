Freelancia Escrow Smart Contract Outline (Solana)


Purpose

The escrow Solana program secures payments for Freelancia job proposals, locking funds until the client approves job completion or a dispute is resolved. It integrates with Freelancia's Rust backend (Axum, SQLite) to manage escrow for accepted proposals, release payments, and handle disputes on the Solana blockchain.
Program Details

File: /programs/escrow/src/lib.rs
Language: Rust (Anchor ^0.31.1)
Blockchain: Solana
Currency: SOL (lamports, 1 SOL = 10^9 lamports)
Framework: Anchor for simplified account management and serialization

Accounts and Enums

EscrowAccount (account):
proposal_id: u64, Backend proposal ID.
job_id: u64, Backend job ID.
client: Pubkey, Job poster.
freelancer: Pubkey, Proposal submitter.
amount: u64, Escrow amount (lamports).
status: EscrowStatus, Escrow state.
disputed: bool, Dispute flag.
authority: Pubkey, Dispute resolver.


EscrowStatus (enum): Funded, Released, Disputed, Resolved.

Instructions

create_escrow(proposal_id, job_id, amount):
Caller: Client (signer).
Accounts: escrow (PDA, init), client, freelancer, authority, system_program.
Action: Initializes escrow PDA, transfers amount lamports from client, sets status=Funded.
Emits: EscrowCreated.


release_payment():
Caller: Client (signer).
Accounts: escrow (PDA), client, freelancer.
Action: Transfers amount to freelancer, sets status=Released.
Emits: PaymentReleased.


raise_dispute():
Caller: Client or Freelancer (signer).
Accounts: escrow (PDA), payer (client/freelancer), client, freelancer.
Action: Sets status=Disputed, disputed=true.
Emits: DisputeRaised.


resolve_dispute(amount_to_recipient):
Caller: Authority (signer).
Accounts: escrow (PDA), authority, client, recipient, freelancer.
Action: Transfers amount_to_recipient to recipient (client/freelancer), remainder to client, sets status=Resolved.
Emits: DisputeResolved.



Events

EscrowCreated(proposal_id, job_id, client, freelancer, amount):
Triggered: When escrow is created.
Use: Backend tracks new escrows.


PaymentReleased(proposal_id, freelancer, amount):
Triggered: When client releases payment.
Use: Notify freelancer of payment.


DisputeRaised(proposal_id, raiser):
Triggered: When client/freelancer raises a dispute.
Use: Alert authority for resolution.


DisputeResolved(proposal_id, recipient, amount):
Triggered: When authority resolves dispute.
Use: Notify parties of outcome.



Integration with Freelancia Backend

Proposal Acceptance (PATCH /api/proposals/:id):
Backend calls create_escrow when status=accepted.
Inputs: proposal_id, job_id, freelancer (wallet_address from users), amount (bid_amount in lamports).
Uses anchor_client to send transaction.


Job Completion:
Client calls release_payment via frontend (e.g., Phantom wallet) or backend API.


Disputes:
Client/freelancer calls raise_dispute via frontend/backend.
Authority resolves via resolve_dispute (off-chain coordination).


Events:
Backend listens for events using anchor_client or solana-client to parse transaction logs, updating proposals table.



Security Considerations

Account Validation: Anchor constraints (has_one, signer, seeds) ensure correct accounts.
Fund Safety: Lamports are locked in PDA, only transferable with valid status.
Access Control: Client-only for create_escrow/release_payment, authority-only for resolve_dispute.
Error Handling: Custom EscrowError for invalid states (e.g., Disputed, InvalidStatus).
Solana-Specific: PDA prevents double-spending; try_borrow_mut_lamports ensures safe transfers.

Future Improvements

Multi-Sig Authority: Replace authority with a multi-sig program.
Token Support: Use SPL tokens (e.g., USDC) via anchor-spl.
Timeout Mechanism: Auto-release/refund after a deadline.
Pagination: For querying escrow accounts.
Upgradability: Use Anchor’s upgradable program support.



