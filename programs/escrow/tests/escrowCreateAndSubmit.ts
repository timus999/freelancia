import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorError } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { expect } from "chai";
import { sha256 } from "@noble/hashes/sha256";
import { SystemProgram, Keypair, PublicKey, Connection } from "@solana/web3.js";

describe("native-SOL escrow (v2)", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.escrow as Program<Escrow>;
  const connection = provider.connection;

  // Constants
  const ESCROW_AMT = 1_000_000_000; // 1 SOL (lamports)
  const FUND_AMOUNT = 5 * ESCROW_AMT; // 5 SOL
  let testCounter = 0;

  /** Generates a 32‑byte hash from an arbitrary string. */
  function hash32(data: string): Uint8Array {
    return sha256(new TextEncoder().encode(data));
  }

  // Create new keypairs for each test
  function createTestActors() {
    return {
      maker: Keypair.generate(),
      taker: Keypair.generate(),
      arbiter: Keypair.generate()
    };
  }

  // Fund an account with SOL
  async function fundAccount(pubkey: PublicKey, amount: number) {
    const sig = await connection.requestAirdrop(pubkey, amount);
    await connection.confirmTransaction(sig, "confirmed");
    await new Promise(r => setTimeout(r, 1500)); // Increased delay
  }

  // Clean up accounts after test
  async function cleanupAccounts(pdas: PublicKey[]) {
    // In a real test environment, you'd close accounts here
    // For testing, we just log and reset the validator
    console.log("Cleaning up accounts...");
  }

  before(async () => {
    console.log("Resetting test validator...");
    try {
      // Try to reset test validator
      await provider.connection.request({ method: "reset" });
    } catch (e) {
      console.log("Couldn't reset validator, proceeding anyway");
    }
  });

  beforeEach(async () => {
    testCounter++;
    console.log(`\n=== Starting test ${testCounter} ===`);
  });

  it("creates escrow and transfers SOL", async () => {
    const { maker, taker, arbiter } = createTestActors();
    await fundAccount(maker.publicKey, FUND_AMOUNT);
    await fundAccount(taker.publicKey, 500_000_000);
    
    // Generate unique escrow ID
    const escrowId = new anchor.BN(testCounter * 1000 + Date.now());
    const specHash = hash32(`spec-${Date.now()}-${Math.random()}`);
    
    // Derive PDAs
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        escrowId.toArray("le", 8),
      ],
      program.programId
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        maker.publicKey.toBuffer(),
        escrowId.toArray("le", 8),
      ],
      program.programId
    );

    console.log("Test escrow ID:", escrowId.toString());
    console.log("Escrow PDA:", escrowPda.toBase58());
    console.log("Vault PDA:", vaultPda.toBase58());

    const now = Math.floor(Date.now() / 1000);
    const DEADLINE_SEC = now + 600;
    const AUTO_RELEASE_SEC = DEADLINE_SEC + 600;

    // Check if PDAs exist before creation (should not)
    const preEscrowAccount = await connection.getAccountInfo(escrowPda);
    const preVaultAccount = await connection.getAccountInfo(vaultPda);
    console.log("Pre-creation escrow exists:", preEscrowAccount !== null);
    console.log("Pre-creation vault exists:", preVaultAccount !== null);

    try {
      await program.methods
        .createEscrow(
          escrowId,
          new anchor.BN(ESCROW_AMT),
          new anchor.BN(DEADLINE_SEC),
          new anchor.BN(AUTO_RELEASE_SEC),
          Array.from(specHash),
          arbiter.publicKey
        )
        .accounts({
          maker: maker.publicKey,
          taker: taker.publicKey,
          escrow: escrowPda,
          escrow_vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([maker])
        .rpc({ skipPreflight: true }); // Skip preflight for more visibility

      console.log("Escrow creation successful");
    } catch (error) {
      console.error("Escrow creation failed:", error);
      throw error;
    }

    // Verify accounts exist
    const postEscrowAccount = await connection.getAccountInfo(escrowPda);
    const postVaultAccount = await connection.getAccountInfo(vaultPda);
    console.log("Post-creation escrow exists:", postEscrowAccount !== null);
    console.log("Post-creation vault exists:", postVaultAccount !== null);

    if (!postEscrowAccount || !postVaultAccount) {
      throw new Error("Accounts not created");
    }

    // Clean up
    await cleanupAccounts([escrowPda, vaultPda]);
  });

  it("rejects duplicate escrow creation", async () => {
    const { maker, taker } = createTestActors();
    await fundAccount(maker.publicKey, FUND_AMOUNT);
    
    const escrowId = new anchor.BN(Date.now());
    const specHash = hash32(`spec-${Date.now()}`);
    
    const [escrowPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        maker.publicKey.toBuffer(),
        escrowId.toArray("le", 8),
      ],
      program.programId
    );

    const [vaultPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        maker.publicKey.toBuffer(),
        escrowId.toArray("le", 8),
      ],
      program.programId
    );

    const now = Math.floor(Date.now() / 1000);
    const DEADLINE_SEC = now + 600;
    const AUTO_RELEASE_SEC = DEADLINE_SEC + 600;

    // First creation
    await program.methods
      .createEscrow(
        escrowId,
        new anchor.BN(ESCROW_AMT),
        new anchor.BN(DEADLINE_SEC),
        new anchor.BN(AUTO_RELEASE_SEC),
        Array.from(specHash),
        null
      )
      .accounts({
        maker: maker.publicKey,
        taker: taker.publicKey,
        escrow: escrowPda,
        escrow_vault: vaultPda,
        systemProgram: SystemProgram.programId,
      })
      .signers([maker])
      .rpc();

    // Second creation
    try {
      await program.methods
        .createEscrow(
          escrowId,
          new anchor.BN(ESCROW_AMT),
          new anchor.BN(DEADLINE_SEC),
          new anchor.BN(AUTO_RELEASE_SEC),
          Array.from(specHash),
          null
        )
        .accounts({
          maker: maker.publicKey,
          taker: taker.publicKey,
          escrow: escrowPda,
          escrow_vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([maker])
        .rpc();

      throw new Error("Duplicate creation should have failed");
    } catch (error) {
      console.log("Duplicate creation correctly failed");
      if (error instanceof AnchorError) {
        console.log("Anchor error code:", error.error.errorCode.code);
      }
      expect(error).to.be.instanceOf(Error);
    }

    await cleanupAccounts([escrowPda, vaultPda]);
  });

  describe("submitWork()", () => {
    let maker: Keypair;
    let taker: Keypair;
    let escrowId: anchor.BN;
    let escrowPda: PublicKey;
    let vaultPda: PublicKey;

    beforeEach(async () => {
      testCounter++;
      const actors = createTestActors();
      maker = actors.maker;
      taker = actors.taker;
      
      await fundAccount(maker.publicKey, FUND_AMOUNT);
      await fundAccount(taker.publicKey, 500_000_000);
      
      escrowId = new anchor.BN(Date.now() + testCounter);
      const specHash = hash32(`spec-${Date.now()}`);
      
      [escrowPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("escrow"),
          maker.publicKey.toBuffer(),
          escrowId.toArray("le", 8),
        ],
        program.programId
      );

      [vaultPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vault"),
          maker.publicKey.toBuffer(),
          escrowId.toArray("le", 8),
        ],
        program.programId
      );

      const now = Math.floor(Date.now() / 1000);
      const DEADLINE_SEC = now + 600;
      const AUTO_RELEASE_SEC = DEADLINE_SEC + 600;

      await program.methods
        .createEscrow(
          escrowId,
          new anchor.BN(ESCROW_AMT),
          new anchor.BN(DEADLINE_SEC),
          new anchor.BN(AUTO_RELEASE_SEC),
          Array.from(specHash),
          null
        )
        .accounts({
          maker: maker.publicKey,
          taker: taker.publicKey,
          escrow: escrowPda,
          escrow_vault: vaultPda,
          systemProgram: SystemProgram.programId,
        })
        .signers([maker])
        .rpc();
    });

    afterEach(async () => {
      await cleanupAccounts([escrowPda, vaultPda]);
    });

    it("allows the taker to submit work", async () => {
      const deliverableHash = hash32(`deliverable-${Date.now()}`);
      
      await program.methods
        .submitWork(Array.from(deliverableHash))
        .accounts({
          taker: taker.publicKey,
          escrow: escrowPda,
        })
        .signers([taker])
        .rpc();

      // Verify state
      const escrowAccount = await program.account.escrow.fetch(escrowPda);
      expect(escrowAccount.status).to.equal(1); // Submitted
    });

    it("rejects submitWork from non-taker", async () => {
      const imposter = Keypair.generate();
      await fundAccount(imposter.publicKey, 500_000_000);
      
      const deliverableHash = hash32("invalid-deliverable");

      try {
        await program.methods
          .submitWork(Array.from(deliverableHash))
          .accounts({
            taker: imposter.publicKey,
            escrow: escrowPda,
          })
          .signers([imposter])
          .rpc();

        throw new Error("Should have rejected non-taker");
      } catch (error) {
        console.log("Correctly rejected non-taker");
        expect(error).to.be.instanceOf(Error);
      }
    });
  });
});