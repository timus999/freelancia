// // tests/arbiterResolve.ts
// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import { assert } from "chai";
// import { Escrow } from "../target/types/escrow";

// const { SystemProgram } = anchor.web3;

// describe("arbiter_resolve", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);

//     const connection = provider.connection;

//     const LAMPORTS_PER_SOL = anchor.web3.LAMPORTS_PER_SOL;

//   const program = anchor.workspace.Escrow as Program<Escrow>;
//   const maker = anchor.web3.Keypair.generate();
//   const taker = anchor.web3.Keypair.generate();
//   const arbiter = anchor.web3.Keypair.generate();
//   const escrowId = new anchor.BN(Date.now());

//   let escrowPda: anchor.web3.PublicKey;
//   let vaultPda: anchor.web3.PublicKey;
//   let vaultBump: number;

//   before(async () => {
//     await provider.connection.confirmTransaction(
//       await provider.connection.requestAirdrop(maker.publicKey, 3e9),
//       "confirmed"
//     );
//     await provider.connection.confirmTransaction(
//       await provider.connection.requestAirdrop(arbiter.publicKey, 1e9),
//       "confirmed"
//     );

//     [escrowPda] = await anchor.web3.PublicKey.findProgramAddress(
//       [Buffer.from("escrow"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
//       program.programId
//     );

//     [vaultPda, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
//       [Buffer.from("vault"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
//       program.programId
//     );

//         const now = Math.floor(Date.now() / 1000);
//     const DEADLINE_SEC = now + 600;
//     const AUTO_RELEASE_SEC = DEADLINE_SEC + 600;



//     await program.methods
//       .createEscrow(escrowId,
//           new anchor.BN(1.1 * LAMPORTS_PER_SOL),
//            new anchor.BN(DEADLINE_SEC),
//             new anchor.BN(AUTO_RELEASE_SEC),
//               new Array(32).fill(1),
//                arbiter.publicKey)
//       .accounts({
//         maker: maker.publicKey,
//         taker: taker.publicKey,
//         escrow: escrowPda,
//         vault: vaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([maker])
//       .rpc();

//       await new Promise((res) => setTimeout(res, 2000));


//       const vaultBalance = await connection.getBalance(vaultPda);
//     console.log("Vault balance:", vaultBalance / LAMPORTS_PER_SOL, "SOL");

//     const vaultRent = await connection.getMinimumBalanceForRentExemption(0);
// const actualSpendable = vaultBalance - vaultRent;
// console.log("Spendable:", actualSpendable / LAMPORTS_PER_SOL);

// ;

//     // Manually mark as disputed for test
//     await program.methods
//       .raiseDispute(new Array(32).fill(2))
//       .accounts({
//         caller: maker.publicKey,
//         escrow: escrowPda,
//       })
//       .signers([maker])
//       .rpc();
//   });

//   it("allows arbiter to resolve dispute and split funds", async () => {
//     const takerBalanceBefore = await provider.connection.getBalance(taker.publicKey);
//     const makerBalanceBefore = await provider.connection.getBalance(maker.publicKey);

//     await program.methods
//       .arbiterResolve(new anchor.BN(0.5 * LAMPORTS_PER_SOL), new anchor.BN(0.5 * LAMPORTS_PER_SOL))
//       .accounts({
//         arbiter: arbiter.publicKey,
//         taker: taker.publicKey,
//         maker: maker.publicKey,
//         escrow: escrowPda,
//         vault: vaultPda,
//         systemProgram: SystemProgram.programId,
//       })
//       .signers([arbiter])
//       .rpc();

//     const takerBalanceAfter = await provider.connection.getBalance(taker.publicKey);
//     const makerBalanceAfter = await provider.connection.getBalance(maker.publicKey);

//     assert.ok(takerBalanceAfter > takerBalanceBefore, "Taker received funds");
//     assert.ok(makerBalanceAfter > makerBalanceBefore, "Maker received refund");

//     const escrow = await program.account.escrow.fetch(escrowPda);
//     assert.equal(escrow.status, 2, "Escrow marked completed");
//     assert.equal(escrow.amountReleased.toNumber(), 500_000_000);
//     assert.equal(escrow.amountRefunded.toNumber(), 500_000_000);
//   });
// });
