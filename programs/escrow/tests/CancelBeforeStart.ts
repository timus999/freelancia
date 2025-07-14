// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { Escrow } from "../target/types/escrow";
// import { assert } from "chai";
// import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";

// describe("cancel_before_start", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//   const program = anchor.workspace.Escrow as Program<Escrow>;
//   let escrowPda: PublicKey;
//   let vaultPda: PublicKey;
//   let vaultBump: number;
//   const maker = provider.wallet;
//   const taker = anchor.web3.Keypair.generate();
//   const escrowId = new anchor.BN(Date.now());

//   const amount = 1 * LAMPORTS_PER_SOL;
//   const deadline = Math.floor(Date.now() / 1000) + 3600;
//   const autoRelease = deadline + 3600;

//   it("successfully cancels an active escrow before work starts", async () => {
//     // Derive PDAs
//        const [escrowPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("escrow"),
//         maker.publicKey.toBuffer(),
//         escrowId.toArrayLike(Buffer, "le", 8)
//       ],
//       program.programId
//     );

//     const [vaultPda] = PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("vault"),
//         maker.publicKey.toBuffer(),
//         escrowId.toArrayLike(Buffer, "le", 8),
//       ],
//       program.programId
//     );


//     // Call create_escrow
//     await program.methods
//       .createEscrow(escrowId,
//          new anchor.BN(amount),
//        new anchor.BN(deadline),
//         new anchor.BN(autoRelease),
//          Array(32).fill(1),
//           null)
//       .accounts({
//         maker: maker.publicKey,
//         taker: taker.publicKey,
//         escrow: escrowPda,
//         vault: vaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     const beforeBalance = await provider.connection.getBalance(maker.publicKey);

//     // Cancel escrow
//     await program.methods
//       .cancelBeforeStart()
//       .accounts({
//         maker: maker.publicKey,
//         escrow: escrowPda,
//         vault: vaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .rpc();

//     const afterBalance = await provider.connection.getBalance(maker.publicKey);
//     assert.ok(afterBalance > beforeBalance);

//     const escrow = await program.account.escrow.fetch(escrowPda);
//     assert.equal(escrow.status, 4); // Cancelled status
//     assert.equal(escrow.amountRefunded.toNumber(), amount);
//   });
// });
