// import * as anchor from "@coral-xyz/anchor";
// import { Program, AnchorError } from "@coral-xyz/anchor";
// import { Escrow } from "../target/types/escrow";
// import { expect } from "chai";
// const sha256_1 = require("@noble/hashes/sha256");

// describe("native‑SOL escrow (v2)", () => {
//   const provider = anchor.AnchorProvider.env();
//   anchor.setProvider(provider);
//   const program = anchor.workspace.escrow as Program<Escrow>;
//   const connection = provider.connection;

//   const maker = provider.wallet;
//   const takerKp = anchor.web3.Keypair.generate();
//   const arbiterKp = anchor.web3.Keypair.generate();

//   const ESCROW_AMT = 1_000_000_000; // 1 SOL
//   let ESCROW_COUNTER = 100n; // Global counter for unique escrow IDs
//   let escrowPda: anchor.web3.PublicKey;
//   let vaultPda: anchor.web3.PublicKey;
//   let escrowBump: number;
//   let rentLamports = 0;

//   function hash32(data: string): Uint8Array {
//     return (0, sha256_1.sha256)(new TextEncoder().encode(data));
//   }

//   const specHash = hash32("helloworld");

//   before(async () => {
//     const escrowId = ESCROW_COUNTER++;
//     const now = Math.floor(Date.now() / 1_000);
//     const DEADLINE_SEC = now + 60;
//     const AUTO_RELEASE_SEC = DEADLINE_SEC + 60;

//     [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("escrow"),
//         maker.publicKey.toBuffer(),
//         new anchor.BN(escrowId.toString()).toArrayLike(Buffer, "le", 8),
//       ],
//       program.programId
//     );

//     [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
//       [
//         Buffer.from("vault"),
//         maker.publicKey.toBuffer(),
//         new anchor.BN(escrowId.toString()).toArrayLike(Buffer, "le", 8),
//       ],
//       program.programId
//     );

//     const sigs = await Promise.all([
//       provider.connection.requestAirdrop(maker.publicKey, 5 * ESCROW_AMT),
//       provider.connection.requestAirdrop(takerKp.publicKey, 500_000_000),
//     ]);
//     await Promise.all(
//       sigs.map((sig) => provider.connection.confirmTransaction(sig, "confirmed"))
//     );

//     await new Promise((r) => setTimeout(r, 1_000));

//     rentLamports = await provider.connection.getMinimumBalanceForRentExemption(
//       program.account.escrow.size,
//       "confirmed"
//     );
//   });

//   it("creates escrow and transfers SOL", async () => {
//     const makerBalBefore = await provider.connection.getBalance(maker.publicKey, "confirmed");

//     console.log("Escrow PDA:", escrowPda.toBase58());
//     console.log("Vault PDA:", vaultPda.toBase58());

//     await program.methods
//       .createEscrow(
//         new anchor.BN(ESCROW_COUNTER - 1n), // Use the ID from before
//         new anchor.BN(ESCROW_AMT),
//         new anchor.BN(Math.floor(Date.now() / 1_000) + 60),
//         new anchor.BN(Math.floor(Date.now() / 1_000) + 120),
//         Array.from(specHash),
//         arbiterKp.publicKey
//       )
//       .accounts({
//         maker: maker.publicKey,
//         taker: takerKp.publicKey,
//         escrow: escrowPda,
//         escrow_vault: vaultPda,
//         systemProgram: anchor.web3.SystemProgram.programId,
//       })
//       .rpc();

//     const metadataAccount = await connection.getAccountInfo(escrowPda);
//     console.log("Metadata account exists:", metadataAccount !== null);
//     const vaultAccount = await connection.getAccountInfo(vaultPda);
//     console.log("Vault account exists:", vaultAccount !== null);

//     const e = await program.account.escrow.fetch(escrowPda);
//     expect(e.bump).to.equal(escrowBump);
//     expect(e.maker.toBase58()).to.equal(maker.publicKey.toBase58());
//     expect(e.taker.toBase58()).to.equal(takerKp.publicKey.toBase58());
//     expect(Number(e.amountTotal)).to.equal(ESCROW_AMT);
//     expect(Number(e.amountReleased)).to.equal(0);
//     expect(e.status).to.equal(0);
//     expect(new Uint8Array(e.specHash)).to.deep.equal(specHash);

//     const escrowBal = await provider.connection.getBalance(escrowPda, "confirmed");
//     const rentPaid = escrowBal - ESCROW_AMT;
//     const minRent = rentLamports;
//     const tolerance = 60_000;
//     expect(rentPaid).to.be.at.least(minRent);
//     expect(rentPaid - minRent).to.be.lessThan(tolerance);
//   });

//   it("rejects second escrow with same PDA", async () => {
//     try {
//       await program.methods
//         .createEscrow(
//           new anchor.BN(ESCROW_COUNTER - 1n), // Same ID
//           new anchor.BN(ESCROW_AMT),
//           new anchor.BN(Math.floor(Date.now() / 1_000) + 60),
//           new anchor.BN(Math.floor(Date.now() / 1_000) + 120),
//           Array.from(specHash),
//           null
//         )
//         .accounts({
//           maker: maker.publicKey,
//           taker: takerKp.publicKey,
//           escrow: escrowPda,
//           escrow_vault: vaultPda,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();
//       throw new Error("duplicate escrow unexpectedly succeeded");
//     } catch (err: any) {
//       expect(err.message).to.match(/already in use|initialized/i);
//     }
//   });

//   describe("submitWork()", () => {
//     beforeEach(async () => {
//       const escrowId = ESCROW_COUNTER++;
//       const now = Math.floor(Date.now() / 1_000);
//       const DEADLINE_SEC = now + 60;
//       const AUTO_RELEASE_SEC = DEADLINE_SEC + 60;

//       [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
//         [
//           Buffer.from("escrow"),
//           maker.publicKey.toBuffer(),
//           new anchor.BN(escrowId.toString()).toArrayLike(Buffer, "le", 8),
//         ],
//         program.programId
//       );

//       [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
//         [
//           Buffer.from("vault"),
//           maker.publicKey.toBuffer(),
//           new anchor.BN(escrowId.toString()).toArrayLike(Buffer, "le", 8),
//         ],
//         program.programId
//       );

//       await program.methods
//         .createEscrow(
//           new anchor.BN(escrowId),
//           new anchor.BN(ESCROW_AMT),
//           new anchor.BN(DEADLINE_SEC),
//           new anchor.BN(AUTO_RELEASE_SEC),
//           Array.from(specHash),
//           arbiterKp.publicKey
//         )
//         .accounts({
//           maker: maker.publicKey,
//           taker: takerKp.publicKey,
//           escrow: escrowPda,
//           escrow_vault: vaultPda,
//           systemProgram: anchor.web3.SystemProgram.programId,
//         })
//         .rpc();
//     });

//     it("allows the taker to submit work (happy path)", async () => {
//       const { deadline } = await program.account.escrow.fetch(escrowPda);
//       const now = Math.floor(Date.now() / 1_000);
//       expect(now).to.be.lessThan(Number(deadline));

//       const deliverableHash = hash32("final‑deliverable‑v1");

//       await program.methods
//         .submitWork(Array.from(deliverableHash))
//         .accounts({
//           taker: takerKp.publicKey,
//           escrow: escrowPda,
//         })
//         .signers([takerKp])
//         .rpc();

//       const e = await program.account.escrow.fetch(escrowPda);
//       expect(e.status).to.equal(1);
//       expect(new Uint8Array(e.deliverableHash)).to.deep.equal(deliverableHash);
//     });

//     it("rejects submitWork from a non‑taker account", async () => {
//       const imposter = anchor.web3.Keypair.generate();
//       const badHash = hash32("unauthorised‑attempt");

//       await provider.connection.requestAirdrop(imposter.publicKey, 1_000_000_000);
//       await new Promise((r) => setTimeout(r, 1_000));

//       try {
//         await program.methods
//           .submitWork(Array.from(badHash))
//           .accounts({ taker: imposter.publicKey, escrow: escrowPda })
//           .signers([imposter])
//           .rpc();
//         throw new Error("unauthorised submitWork unexpectedly succeeded");
//       } catch (err) {
//         const anchorErr = err as AnchorError;
//         expect(anchorErr.error.errorCode.code).to.equal("Unauthorized");
//         expect(anchorErr.error.errorCode.number).to.equal(6004);
//         expect(anchorErr.error.errorMessage).to.match(/Unauthorized access/i);
//       }
//     });
//   });
// });