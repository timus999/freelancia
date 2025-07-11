import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SystemProgram, PublicKey, Keypair } from "@solana/web3.js";
import { assert, expect } from "chai";
import { Escrow } from "../target/types/escrow";
const sha256_1 = require("@noble/hashes/sha256");

const LAMPORTS = anchor.web3.LAMPORTS_PER_SOL / 10; // 0.1 SOL

function hash32(data: string): Uint8Array {
  return (0, sha256_1.sha256)(new TextEncoder().encode(data));
}

async function airdrop(provider: anchor.AnchorProvider, pubkey: PublicKey, amount: number) {
  const sig = await provider.connection.requestAirdrop(pubkey, amount);
  await provider.connection.confirmTransaction(sig, "confirmed");
}

describe("approve_work", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const connection = provider.connection;
  const program = anchor.workspace.escrow as Program<Escrow>;

  const maker = anchor.web3.Keypair.generate();
  const taker = anchor.web3.Keypair.generate();
  const stranger = anchor.web3.Keypair.generate();
  const escrowId = new anchor.BN(777);

  const amount = new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL);
  const deadline = new anchor.BN(Math.floor(Date.now() / 1000) + 86400);
  const autoReleaseAt = new anchor.BN(Math.floor(Date.now() / 1000) + 172800);
  const specHash = hash32("dummy");

  let escrowPda: anchor.web3.PublicKey;
  let vaultPda: anchor.web3.PublicKey;
  let bump: number;

  before(async () => {
    const fundAmount = 2 * anchor.web3.LAMPORTS_PER_SOL;

    await airdrop(provider, maker.publicKey, fundAmount);
    await airdrop(provider, taker.publicKey, fundAmount);
    await airdrop(provider, stranger.publicKey, fundAmount); // Fund stranger

    await new Promise(resolve => setTimeout(resolve, 2000));

    [escrowPda, bump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        escrowId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        maker.publicKey.toBuffer(),
        escrowId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    console.log("Program ID:", program.programId.toBase58());
    console.log("Maker:", maker.publicKey.toBase58());
    console.log("Metadata PDA:", escrowPda.toBase58());
    console.log("Vault PDA:", vaultPda.toBase58());
    console.log("Vault PDA exists before:", (await connection.getAccountInfo(vaultPda)) !== null);

    try {
      await program.methods
        .createEscrow(
          escrowId,
          amount,
          deadline,
          autoReleaseAt,
          Array.from(specHash),
          null
        )
        .accounts({
          maker: maker.publicKey,
          taker: taker.publicKey,
          escrow: escrowPda,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([maker])
        .rpc();

      console.log("Escrow created successfully");

      const metadataAccount = await connection.getAccountInfo(escrowPda);
      console.log("Metadata account exists:", metadataAccount !== null);

      const vaultAccount = await connection.getAccountInfo(vaultPda);
      console.log("Vault account exists:", vaultAccount !== null);
      console.log("Vault balance:", await connection.getBalance(vaultPda));

      const deliverableHash = hash32("dummy");
      await program.methods
        .submitWork(Array.from(deliverableHash))
        .accounts({
          taker: taker.publicKey,
          escrow: escrowPda,
        })
        .signers([taker])
        .rpc();

      console.log("Work submitted successfully");
    } catch (error) {
      console.error("Setup failed:", error);
      if (error.logs) {
        console.log("Transaction logs:", error.logs);
      }
      throw error;
    }
  });

  it("transfers remaining funds and marks escrow as Completed", async () => {
    const initialEscrowBalance = await provider.connection.getBalance(vaultPda);
    const initialTakerBalance = await provider.connection.getBalance(taker.publicKey);

    try {
      await program.methods
        .approveWork()
        .accounts({
          maker: maker.publicKey,
          taker: taker.publicKey,
          escrow: escrowPda,
          vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([maker])
        .preInstructions([
          anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({ units: 300_000 }),
        ])
        .rpc();
    } catch (error) {
      console.error("ApproveWork failed:", error);
      if (error.logs) {
        console.log("Transaction logs:", error.logs);
      }
      throw error;
    }

    const escrowAccount = await program.account.escrow.fetch(escrowPda);

    const finalEscrowBalance = await provider.connection.getBalance(vaultPda);
    const finalTakerBalance = await provider.connection.getBalance(taker.publicKey);

    expect(finalTakerBalance).to.be.greaterThan(
      initialTakerBalance,
      "Taker should receive the payment"
    );
    expect(finalEscrowBalance).to.be.lessThan(
      initialEscrowBalance,
      "Escrow balance should decrease"
    );
    expect(escrowAccount.status).to.equal(2, "Escrow status should be completed");
    expect(escrowAccount.amountReleased.toString()).to.equal(
      amount.toString(),
      "Amount released should equal total amount"
    );
    expect(escrowAccount.completedAt.toNumber()).to.be.greaterThan(
      0,
      "Completed at timestamp should be set"
    );
  });

  it("fails if someone other than the maker calls approve_work", async () => {
  const freshMaker = anchor.web3.Keypair.generate();
  const freshEscrowId = new anchor.BN(999); // different ID
  const fundAmount = 2 * anchor.web3.LAMPORTS_PER_SOL;

  await airdrop(provider, freshMaker.publicKey, fundAmount);

  const [freshEscrowPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("escrow"),
      freshMaker.publicKey.toBuffer(),
      freshEscrowId.toArrayLike(Buffer, "le", 8),
    ],
    program.programId
  );

  const [freshVaultPda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("vault"),
      freshMaker.publicKey.toBuffer(),
      freshEscrowId.toArrayLike(Buffer, "le", 8),
    ],
    program.programId
  );

  const now = Math.floor(Date.now() / 1000);
  const dummyHash = hash32("dummy");

  await program.methods
    .createEscrow(
      freshEscrowId,
      new anchor.BN(LAMPORTS),
      new anchor.BN(now + 60),
      new anchor.BN(now + 3600),
      Array.from(dummyHash),
      null
    )
    .accounts({
      maker: freshMaker.publicKey,
      taker: taker.publicKey,
      escrow: freshEscrowPda,
      vault: freshVaultPda,
      systemProgram: SystemProgram.programId,
    })
    .signers([freshMaker])
    .rpc();

  await program.methods
    .submitWork(Array.from(dummyHash))
    .accounts({
      taker: taker.publicKey,
      escrow: freshEscrowPda,
    })
    .signers([taker])
    .rpc();

  try {
    await program.methods
      .approveWork()
      .accounts({
        maker: stranger.publicKey,
        taker: taker.publicKey,
        escrow: freshEscrowPda,
        vault: freshVaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([stranger])
      .rpc();
    expect.fail("Should have thrown an error");
  } catch (err) {
    expect(err).to.be.instanceOf(anchor.AnchorError);
    expect(err.error.errorCode.code).to.equal("Unauthorized"); // ✅ Now this will be hit
  }
});


  it("fails if escrow.status != Submitted", async () => {
    const freshMaker = anchor.web3.Keypair.generate();
    const fundAmount = 2 * anchor.web3.LAMPORTS_PER_SOL;

    await airdrop(provider, freshMaker.publicKey, fundAmount);

    const [freshEscrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        freshMaker.publicKey.toBuffer(),
        escrowId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const [freshVaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        freshMaker.publicKey.toBuffer(),
        escrowId.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const now = Math.floor(Date.now() / 1000);
    const dummyHash = hash32("dummy");

    await program.methods
      .createEscrow(
        escrowId,
        new anchor.BN(LAMPORTS),
        new anchor.BN(now + 60),
        new anchor.BN(now + 3600),
        Array.from(dummyHash),
        null
      )
      .accounts({
        maker: freshMaker.publicKey,
        taker: taker.publicKey,
        escrow: freshEscrowPda,
        vault: freshVaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([freshMaker])
      .rpc();

    try {
      await program.methods
        .approveWork()
        .accounts({
          maker: freshMaker.publicKey,
          taker: taker.publicKey,
          escrow: freshEscrowPda,
          vault: freshVaultPda, // Fix account key
          systemProgram: SystemProgram.programId,
        })
        .signers([freshMaker])
        .rpc();
      expect.fail("Should have thrown an error");
    } catch (err) {
      expect(err).to.be.instanceOf(anchor.AnchorError);
      expect(err.error.errorCode.code).to.equal("InvalidState");
    }
  });
});