import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { LavaPrograms } from "../target/types/lava_programs";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";
import { expect } from "chai";

describe("lava-programs", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.lavaPrograms as Program<LavaPrograms>;

  // Test accounts
  let authority: Keypair;
  let treasury: Keypair;
  let user1: Keypair;
  let user2: Keypair;

  // Token mints
  let tokenMint: PublicKey;
  let usdcMint: PublicKey;
  let usdtMint: PublicKey;

  // Token accounts
  let user1UsdcAccount: PublicKey;
  let user1UsdtAccount: PublicKey;
  let treasuryUsdcAccount: PublicKey;
  let treasuryUsdtAccount: PublicKey;

  // PDAs
  let presaleConfig: PublicKey;
  let user1Contribution: PublicKey;
  let user2Contribution: PublicKey;

  before(async () => {
    // Generate keypairs
    authority = Keypair.generate();
    treasury = Keypair.generate();
    user1 = Keypair.generate();
    user2 = Keypair.generate();

    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(authority.publicKey, 10 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(treasury.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user1.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 5 * anchor.web3.LAMPORTS_PER_SOL);

    // Create token mints
    tokenMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6 // 6 decimals for presale token
    );

    usdcMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6 // USDC has 6 decimals
    );

    usdtMint = await createMint(
      provider.connection,
      authority,
      authority.publicKey,
      null,
      6 // USDT has 6 decimals
    );

    // Create token accounts
    user1UsdcAccount = await createAccount(
      provider.connection,
      user1,
      usdcMint,
      user1.publicKey
    );

    user1UsdtAccount = await createAccount(
      provider.connection,
      user1,
      usdtMint,
      user1.publicKey
    );

    treasuryUsdcAccount = await createAccount(
      provider.connection,
      treasury,
      usdcMint,
      treasury.publicKey
    );

    treasuryUsdtAccount = await createAccount(
      provider.connection,
      treasury,
      usdtMint,
      treasury.publicKey
    );

    // Mint test tokens to user1
    await mintTo(
      provider.connection,
      authority,
      usdcMint,
      user1UsdcAccount,
      authority,
      1000 * 1e6 // 1000 USDC
    );

    await mintTo(
      provider.connection,
      authority,
      usdtMint,
      user1UsdtAccount,
      authority,
      1000 * 1e6 // 1000 USDT
    );

    // Derive PDAs
    [presaleConfig] = PublicKey.findProgramAddressSync(
      [Buffer.from("presale")],
      program.programId
    );

    [user1Contribution] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_contribution"), user1.publicKey.toBuffer(), presaleConfig.toBuffer()],
      program.programId
    );

    [user2Contribution] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_contribution"), user2.publicKey.toBuffer(), presaleConfig.toBuffer()],
      program.programId
    );
  });

  it("Initializes presale with multiple stages", async () => {
    const now = Math.floor(Date.now() / 1000);
    const stages = [
      {
        stageId: 0,
        tokenPriceUsd: new anchor.BN(100000), // $0.1 per token (6 decimals)
        tokenSupply: new anchor.BN(1000000 * 1e6), // 1M tokens
        tokensSold: new anchor.BN(0),
        startTime: new anchor.BN(now),
        endTime: new anchor.BN(now + 3600), // 1 hour
        minContributionUsd: new anchor.BN(10 * 1e6), // $10 min
        maxContributionUsd: new anchor.BN(1000 * 1e6), // $1000 max
      },
      {
        stageId: 1,
        tokenPriceUsd: new anchor.BN(150000), // $0.15 per token
        tokenSupply: new anchor.BN(500000 * 1e6), // 500K tokens
        tokensSold: new anchor.BN(0),
        startTime: new anchor.BN(now + 3600),
        endTime: new anchor.BN(now + 7200), // 2 hours total
        minContributionUsd: new anchor.BN(10 * 1e6),
        maxContributionUsd: new anchor.BN(1000 * 1e6),
      },
    ];

    await program.methods
      .initializePresale(
        stages,
        new anchor.BN(50000 * 1e6), // $50K soft cap
        new anchor.BN(200000 * 1e6), // $200K hard cap
        new anchor.BN(now),
        new anchor.BN(now + 7200)
      )
      .accounts({
        presaleConfig,
        authority: authority.publicKey,
        treasury: treasury.publicKey,
        tokenMint,
        usdcMint,
        usdtMint,
        systemProgram: SystemProgram.programId,
      })
      .signers([authority])
      .rpc();

    // Verify presale config
    const presaleAccount = await program.account.presaleConfig.fetch(presaleConfig);
    expect(presaleAccount.authority.toString()).to.equal(authority.publicKey.toString());
    expect(presaleAccount.treasury.toString()).to.equal(treasury.publicKey.toString());
    expect(presaleAccount.stages.length).to.equal(2);
    expect(presaleAccount.currentStage).to.equal(0);
    expect(presaleAccount.softCap.toNumber()).to.equal(50000 * 1e6);
    expect(presaleAccount.hardCap.toNumber()).to.equal(200000 * 1e6);
    expect(presaleAccount.finalized).to.be.false;
    expect(presaleAccount.paused).to.be.false;
  });

  it("Allows SOL contributions", async () => {
    const contributionAmount = 1 * anchor.web3.LAMPORTS_PER_SOL; // 1 SOL
    const solPriceUsd = 100 * 1e6; // $100 per SOL

    await program.methods
      .contribute(
        { sol: { amount: new anchor.BN(contributionAmount) } },
        new anchor.BN(solPriceUsd)
      )
      .accounts({
        presaleConfig,
        userContribution: user1Contribution,
        user: user1.publicKey,
        treasury: treasury.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // Verify contribution
    const contribution = await program.account.userContribution.fetch(user1Contribution);
    expect(contribution.user.toString()).to.equal(user1.publicKey.toString());
    expect(contribution.solContributed.toNumber()).to.equal(contributionAmount);
    expect(contribution.totalContributedUsd.toNumber()).to.equal(100 * 1e6); // $100

    // Verify tokens purchased (100 USD / 0.1 USD per token = 1000 tokens)
    expect(contribution.totalTokensPurchased.toNumber()).to.equal(1000 * 1e6);
  });

  it("Allows USDC contributions", async () => {
    const contributionAmount = 50 * 1e6; // 50 USDC

    await program.methods
      .contribute(
        { usdc: { amount: new anchor.BN(contributionAmount) } },
        new anchor.BN(0) // SOL price not needed for USDC
      )
      .accounts({
        presaleConfig,
        userContribution: user1Contribution,
        user: user1.publicKey,
        treasury: treasury.publicKey,
        userTokenAccount: user1UsdcAccount,
        treasuryTokenAccount: treasuryUsdcAccount,
        tokenMint: usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // Verify additional contribution
    const contribution = await program.account.userContribution.fetch(user1Contribution);
    expect(contribution.usdcContributed.toNumber()).to.equal(contributionAmount);
    expect(contribution.totalContributedUsd.toNumber()).to.equal(150 * 1e6); // $100 SOL + $50 USDC

    // Verify additional tokens purchased (50 USD / 0.1 USD per token = 500 tokens)
    expect(contribution.totalTokensPurchased.toNumber()).to.equal(1500 * 1e6); // 1000 + 500
  });

  it("Enforces per-wallet contribution limits", async () => {
    const largeContribution = 1000 * 1e6; // $1000 (at the limit)

    await program.methods
      .contribute(
        { usdc: { amount: new anchor.BN(largeContribution) } },
        new anchor.BN(0)
      )
      .accounts({
        presaleConfig,
        userContribution: user1Contribution,
        user: user1.publicKey,
        treasury: treasury.publicKey,
        userTokenAccount: user1UsdcAccount,
        treasuryTokenAccount: treasuryUsdcAccount,
        tokenMint: usdcMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([user1])
      .rpc();

    // Now user1 is at the $1000 limit, next contribution should fail
    try {
      await program.methods
        .contribute(
          { usdc: { amount: new anchor.BN(1 * 1e6) } }, // Even $1 more should fail
          new anchor.BN(0)
        )
        .accounts({
          presaleConfig,
          userContribution: user1Contribution,
          user: user1.publicKey,
          treasury: treasury.publicKey,
          userTokenAccount: user1UsdcAccount,
          treasuryTokenAccount: treasuryUsdcAccount,
          tokenMint: usdcMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      expect.fail("Should have failed due to exceeding max contribution");
    } catch (error) {
      expect(error.message).to.include("ExceedsMaxContribution");
    }
  });

  it("Allows presale to be paused by authority", async () => {
    await program.methods
      .pausePresale()
      .accounts({
        presaleConfig,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const presaleAccount = await program.account.presaleConfig.fetch(presaleConfig);
    expect(presaleAccount.paused).to.be.true;
  });

  it("Prevents contributions when paused", async () => {
    try {
      await program.methods
        .contribute(
          { sol: { amount: new anchor.BN(anchor.web3.LAMPORTS_PER_SOL) } },
          new anchor.BN(100 * 1e6)
        )
        .accounts({
          presaleConfig,
          userContribution: user2Contribution,
          user: user2.publicKey,
          treasury: treasury.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      expect.fail("Should have failed due to presale being paused");
    } catch (error) {
      expect(error.message).to.include("PresalePaused");
    }
  });

  it("Allows presale to be unpaused by authority", async () => {
    await program.methods
      .unpausePresale()
      .accounts({
        presaleConfig,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const presaleAccount = await program.account.presaleConfig.fetch(presaleConfig);
    expect(presaleAccount.paused).to.be.false;
  });

  it("Prevents unauthorized pause/unpause", async () => {
    try {
      await program.methods
        .pausePresale()
        .accounts({
          presaleConfig,
          authority: user1.publicKey, // Wrong authority
        })
        .signers([user1])
        .rpc();

      expect.fail("Should have failed due to unauthorized access");
    } catch (error) {
      expect(error.message).to.include("Unauthorized");
    }
  });

  it("Allows stage progression", async () => {
    await program.methods
      .updateStage()
      .accounts({
        presaleConfig,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const presaleAccount = await program.account.presaleConfig.fetch(presaleConfig);
    expect(presaleAccount.currentStage).to.equal(1);
  });

  it("Allows presale finalization", async () => {
    await program.methods
      .finalizePresale()
      .accounts({
        presaleConfig,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();

    const presaleAccount = await program.account.presaleConfig.fetch(presaleConfig);
    expect(presaleAccount.finalized).to.be.true;
  });

  it("Prevents contributions after finalization", async () => {
    try {
      await program.methods
        .contribute(
          { sol: { amount: new anchor.BN(anchor.web3.LAMPORTS_PER_SOL) } },
          new anchor.BN(100 * 1e6)
        )
        .accounts({
          presaleConfig,
          userContribution: user2Contribution,
          user: user2.publicKey,
          treasury: treasury.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      expect.fail("Should have failed due to presale being finalized");
    } catch (error) {
      expect(error.message).to.include("PresaleEnded");
    }
  });
});
