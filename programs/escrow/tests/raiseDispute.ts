// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { Escrow } from "../target/types/escrow";
// import { assert } from "chai";
// import { PublicKey, SystemProgram } from "@solana/web3.js";

// describe("raise_dispute", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.Escrow as Program<Escrow>;
//   const maker = provider.wallet;
//   const taker = anchor.web3.Keypair.generate();

//   const createFreshEscrow = async (escrowId: anchor.BN, initialStatus: "active" | "submitted" = "active") => {
//     const [escrowPda] = anchor.web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("escrow"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
//       program.programId
//     );
//     const [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
//       [Buffer.from("vault"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
//       program.programId
//     );

//     // Create
//     await program.methods
//       .createEscrow(
//         escrowId,
//         new anchor.BN(1_000_000),
//         new anchor.BN(Date.now() / 1000 + 5000),
//         new anchor.BN(Date.now() / 1000 + 10000),
//         Array(32).fill(1),
//         null
//       )
//       .accounts({
//         maker: maker.publicKey,
//         taker: taker.publicKey,
//         escrow: escrowPda,
//         vault: vaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     // Optionally submit work
//     if (initialStatus === "submitted") {
//       await program.methods
//         .submitWork(Array(32).fill(2))
//         .accounts({
//           taker: taker.publicKey,
//           escrow: escrowPda,
//         })
//         .signers([taker])
//         .rpc();
//     }

//     return { escrowPda, vaultPda };
//   };

//   before(async () => {
//     // Airdrop to taker
//     const sig = await provider.connection.requestAirdrop(
//       taker.publicKey,
//       1 * anchor.web3.LAMPORTS_PER_SOL
//     );
//     await provider.connection.confirmTransaction(sig);
//   });

//   it("raises dispute successfully (by taker)", async () => {
//     const escrowId = new anchor.BN(10001);
//     const { escrowPda } = await createFreshEscrow(escrowId, "submitted");

//     const evidence = Array(32).fill(3);

//     await program.methods
//       .raiseDispute(evidence)
//       .accounts({
//         caller: taker.publicKey,
//         escrow: escrowPda,
//       })
//       .signers([taker])
//       .rpc();

//     const escrowAcc = await program.account.escrow.fetch(escrowPda);
//     assert.equal(escrowAcc.status, 3); // Disputed
//     assert.deepEqual(escrowAcc.disputeEvidenceUriHash, evidence);
//   });

//   it("fails if unauthorized user tries to raise dispute", async () => {
//     const escrowId = new anchor.BN(10002);
//     const { escrowPda } = await createFreshEscrow(escrowId, "submitted");

//     const stranger = anchor.web3.Keypair.generate();
//     const sig = await provider.connection.requestAirdrop(stranger.publicKey, 1e9);
//     await provider.connection.confirmTransaction(sig);

//     const evidence = Array(32).fill(9);

//     try {
//       await program.methods
//         .raiseDispute(evidence)
//         .accounts({
//           caller: stranger.publicKey,
//           escrow: escrowPda,
//         })
//         .signers([stranger])
//         .rpc();
//       assert.fail("Expected to throw Unauthorized error");
//     } catch (err: any) {
//       assert.equal(err.error.errorCode.code, "Unauthorized");
//     }
//   });

//   it("fails if escrow not in Active or Submitted state", async () => {
//     const escrowId = new anchor.BN(10003);
//     const { escrowPda, vaultPda } = await createFreshEscrow(escrowId, "submitted");

//     // Approve work to make it Completed
//     await program.methods
//       .approveWork()
//       .accounts({
//         maker: maker.publicKey,
//         taker: taker.publicKey,
//         escrow: escrowPda,
//         vault: vaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     const evidence = Array(32).fill(5);

//     try {
//       await program.methods
//         .raiseDispute(evidence)
//         .accounts({
//           caller: maker.publicKey,
//           escrow: escrowPda,
//         })
//         .signers([])
//         .rpc();
//       assert.fail("Expected InvalidState error");
//     } catch (err: any) {
//       assert.equal(err.error.errorCode.code, "InvalidState");
//     }
//   });
// });
