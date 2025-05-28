import anchor from '@coral-xyz/anchor';
import { assert } from 'chai';

// Explicitly set provider config
const provider = anchor.AnchorProvider.local();
anchor.setProvider(provider);

describe('escrow', () => {
  const program = anchor.workspace.Escrow;

  it('Initializes escrow account', async () => {
    const escrowAccount = anchor.web3.Keypair.generate();
    const amount = new anchor.BN(1000);

    try {
      const tx = await program.methods.initialize(amount)
        .accounts({
          escrowAccount: escrowAccount.publicKey,
          user: provider.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([escrowAccount])
        .rpc();

      console.log('\n✅ Transaction signature:', tx);
      
      const account = await program.account.escrowAccount.fetch(escrowAccount.publicKey);
      assert.equal(account.amount.toString(), '1000');
      assert.isTrue(account.isInitialized);
      
      console.log('Escrow initialized successfully!');
    } catch (error) {
      console.error('❌ Error:', error);
      throw error;
    }
  });
});