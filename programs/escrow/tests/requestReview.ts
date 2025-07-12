// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { Escrow } from "../target/types/escrow";
// import { assert } from "chai";
// import { PublicKey, SystemProgram } from "@solana/web3.js";

// describe("request_revision", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);
//   const program = anchor.workspace.Escrow as Program<Escrow>;

//   const maker = provider.wallet;
//   let taker: anchor.web3.Keypair;

//   let escrowPda: PublicKey;
//   let vaultPda: PublicKey;
//   let bump: number;
//   let vaultBump: number;

//   const escrowId = new anchor.BN(99); // Arbitrary ID
//   const amount = anchor.web3.LAMPORTS_PER_SOL;
//   const specHash = new Uint8Array(32); // Empty spec hash for test

//   before(async () => {
//     taker = anchor.web3.Keypair.generate();

//     // Derive PDA
//     [escrowPda, bump] = await PublicKey.findProgramAddressSync(
//       [Buffer.from("escrow"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
//       program.programId
//     );
//     [vaultPda, vaultBump] = await PublicKey.findProgramAddressSync(
//       [Buffer.from("vault"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
//       program.programId
//     );

//     // Airdrop SOL to taker and maker
//     const tx = await provider.connection.requestAirdrop(taker.publicKey, amount);
//     await provider.connection.confirmTransaction(tx);
//     const tx2 = await provider.connection.requestAirdrop(maker.publicKey, amount * 2);
//     await provider.connection.confirmTransaction(tx2);

//     // Create escrow
//     await program.methods
//       .createEscrow(escrowId, 
//         new anchor.BN(amount),
//          new anchor.BN(Date.now() / 1000 + 600),
//           new anchor.BN(Date.now() / 1000 + 1200),
//            Array.from(specHash),
//             null)
//       .accounts({
//         maker: maker.publicKey,
//         taker: taker.publicKey,
//         escrow: escrowPda,
//         vault: vaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([])
//       .rpc();

//     // Taker submits work
//     await program.methods
//       .submitWork(Array.from(new Uint8Array(32))) // empty deliverable hash
//       .accounts({
//         taker: taker.publicKey,
//         escrow: escrowPda,
//       })
//       .signers([taker])
//       .rpc();
//   });

//   it("allows the maker to request a revision", async () => {
//     // ✅ Call request_revision
//     await program.methods
//       .requestRevision()
//       .accounts({
//         maker: maker.publicKey,
//         escrow: escrowPda,
//       })
//       .signers([])
//       .rpc();

//     const escrow = await program.account.escrow.fetch(escrowPda);
//     assert.equal(escrow.status, 0); // EscrowStatus::Active
//     assert.equal(escrow.revisionRequests, 1);
//   });

//   it("fails if status is not Submitted", async () => {
//     try {
//       await program.methods
//         .requestRevision()
//         .accounts({
//           maker: maker.publicKey,
//           escrow: escrowPda,
//         })
//         .rpc();
//       assert.fail("Should not allow revision request in Active state");
//     } catch (err) {
//       const msg = "Invalid escrow state for this operation";
//       assert.equal(err.error.errorMessage, msg);
//     }
//   });

//   it("fails if unauthorized user tries to request revision", async () => {
//     const fakeMaker = anchor.web3.Keypair.generate();


//      await program.methods
//       .submitWork(Array.from(new Uint8Array(32))) // empty deliverable hash
//       .accounts({
//         taker: taker.publicKey,
//         escrow: escrowPda,
//       })
//       .signers([taker])
//       .rpc();

//     try {
//       await program.methods
//         .requestRevision()
//         .accounts({
//           maker: fakeMaker.publicKey,
//           escrow: escrowPda,
//         })
//         .signers([fakeMaker])
//         .rpc();
//       assert.fail("Unauthorized should not succeed");
//     } catch (err) {
//       const msg = "Unauthorized access";
//       assert.equal(err.error.errorMessage, msg);
//     }
//   });
// });
