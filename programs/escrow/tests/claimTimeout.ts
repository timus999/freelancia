import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { assert } from "chai";
import { LAMPORTS_PER_SOL, PublicKey, SystemProgram } from "@solana/web3.js";

describe("claim_timeout", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;
  const maker = provider.wallet;
  const taker = anchor.web3.Keypair.generate();
  
  const amount = new anchor.BN(1 * LAMPORTS_PER_SOL);

  // Helper to get current blockchain time
  async function getCurrentTime(): Promise<number> {
    const clock = await program.provider.connection.getAccountInfo(
      new PublicKey("SysvarC1ock11111111111111111111111111111111")
    );
    if (!clock?.data) throw new Error("Clock sysvar not found");
    return Number(clock.data.readBigUInt64LE(32));
  }

  it("maker claims refund after deadline (ACTIVE -> CANCELLED)", async () => {
    const escrowId = new anchor.BN(Date.now());
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // const currentTime = await getCurrentTime();
    const currentTime = Math.floor(Date.now() / 1000);
    const deadline = currentTime + 2; // 2 seconds in future
    const autoRelease = deadline + 3600; // 1 hour after deadline

    // Create escrow with valid times
    await program.methods
      .createEscrow(
        escrowId,
        amount,
        new anchor.BN(deadline),
        new anchor.BN(autoRelease),
        Array(32).fill(1),
        null
      )
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        escrow: escrowPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Wait for deadline to pass (real-time wait)
    await new Promise(resolve => setTimeout(resolve, 4000));

    const beforeBalance = await provider.connection.getBalance(maker.publicKey);

    await program.methods
      .claimTimeout()
      .accounts({
        claimant: maker.publicKey,
        escrow: escrowPda,
        vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const afterBalance = await provider.connection.getBalance(maker.publicKey);
    const escrow = await program.account.escrow.fetch(escrowPda);

    assert.ok(afterBalance > beforeBalance, "Maker should receive refund");
    assert.equal(escrow.status, 4, "Escrow should be cancelled");
    assert.equal(escrow.amountRefunded.toString(), amount.toString(), "Full amount should be refunded");
  });

  it("taker claims release after autoReleaseAt (SUBMITTED -> COMPLETED)", async () => {
    const escrowId = new anchor.BN(Date.now() + 1);
    const [newescrowPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [newvaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), maker.publicKey.toBuffer(), escrowId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    // const currentTime = await getCurrentTime();
    const currentTime = Math.floor(Date.now() / 1000);
    const deadline = currentTime + 2; // 2 seconds in future
    const autoRelease = deadline + 2; // 2 seconds after deadline

    // Create escrow with valid times
    await program.methods
      .createEscrow(
        escrowId,
        amount,
        new anchor.BN(deadline),
        new anchor.BN(autoRelease),
        Array(32).fill(1),
        null
      )
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        escrow: newescrowPda,
        vault: newvaultPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Submit work immediately
    await program.methods
      .submitWork(Array(32).fill(1))
      .accounts({
        taker: taker.publicKey,
        escrow: newescrowPda,
      })
      .signers([taker])
      .rpc();

    // Wait for auto-release to pass
    await new Promise(resolve => setTimeout(resolve, 5000));

    const beforeBalance = await provider.connection.getBalance(taker.publicKey);

    await program.methods
      .claimTimeout()
      .accounts({
        claimant: taker.publicKey,
        escrow: newescrowPda,
        vault: newvaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([taker])
      .rpc();

    const afterBalance = await provider.connection.getBalance(taker.publicKey);
    const escrow = await program.account.escrow.fetch(newescrowPda);

    assert.ok(afterBalance > beforeBalance, "Taker should receive funds");
    assert.equal(escrow.status, 2, "Escrow should be completed");
    assert.equal(escrow.amountReleased.toString(), amount.toString(), "Full amount should be released");
  });
});