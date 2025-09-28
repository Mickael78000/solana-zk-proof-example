import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CommitRevealDapp } from "../../solana-commit-reveal/commit-reveal-dapp/target/types/commit_reveal_dapp";
import { expect } from "chai";
import { createHash } from "crypto";

describe("commit-reveal", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.CommitRevealDapp as Program<CommitRevealDapp>;
  const user = anchor.web3.Keypair.generate();
  
  let stateAccount: anchor.web3.PublicKey;
  let commitmentAccount: anchor.web3.PublicKey;
  
  const commitDeadline = Math.floor(Date.now() / 1000) + 3600; // 1 hour from now
  const revealDeadline = commitDeadline + 1800; // 30 minutes after commit
  
  before(async () => {
    // Airdrop SOL to user
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL)
    );
    
    // Derive state PDA
    [stateAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("state")],
      program.programId
    );
  });

  it("Initializes program state", async () => {
    await program.methods
      .initializeState(new anchor.BN(commitDeadline), new anchor.BN(revealDeadline))
      .accounts({
        authority: provider.wallet.publicKey,
      })
      .rpc();

    const state = await program.account.commitRevealState.fetch(stateAccount);
    expect(state.commitDeadline.toNumber()).to.equal(commitDeadline);
    expect(state.revealDeadline.toNumber()).to.equal(revealDeadline);
  });

  it("Successfully commits an order", async () => {
    // Generate commitment
    const orderData = "buy 100 SOL at 50 USDC";
    const secret = "my-secret-12345";
    const dataToHash = Buffer.concat([
      Buffer.from(orderData),
      Buffer.from(secret)
    ]);
    const commitmentHash = createHash('sha256').update(dataToHash).digest();
    
    const commitmentIndex = new anchor.BN(0);
    
    // Derive commitment PDA
    [commitmentAccount] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("commitment"),
        user.publicKey.toBuffer(),
        commitmentIndex.toArrayLike(Buffer, "le", 8)
      ],
      program.programId
    );
    
    await program.methods
      .commitOrder(Buffer.from(commitmentHash), commitmentIndex, null as any)
      .accounts({
        user: user.publicKey,
      })
      .signers([user])
      .rpc();
    
    const commitment = await program.account.commitment.fetch(commitmentAccount);
    expect(commitment.revealed).to.be.false;
    expect(commitment.user.toString()).to.equal(user.publicKey.toString());
  });

  it("Successfully reveals an order", async () => {
    const orderData = "buy 100 SOL at 50 USDC";
    const secret = "my-secret-12345";
    
    await program.methods
      .revealOrder(Buffer.from(orderData), Buffer.from(secret))
      .accounts({
        commitment: commitmentAccount,
        state: stateAccount,
        user: user.publicKey,
      })
      .signers([user])
      .rpc();
    
    const commitment = await program.account.commitment.fetch(commitmentAccount);
    expect(commitment.revealed).to.be.true;
  });
});