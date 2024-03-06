import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import {
  getAssociatedTokenAccount,
  getAuthorizedUserKeypair,
  getClobManagerKeypair,
  getFeeManagerKeypair,
  getUserKeypair,
  getWalletManagerKeypair,
  getWalletManagerTokenAccount,
} from "../utils/wallets";
import { getProgram } from "../utils/program";
import {
  getAuthorizedPDA,
  getMarketPDA,
  getMarketPortfolioPDA,
  getUserPDA,
} from "../utils/pdas";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { beforeEach } from "mocha";

describe("resolved", () => {
  const MINT = new PublicKey("5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes");

  const additionalComputeBudgetInstruction =
      anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
        units: 1_400_000,
      });

  const program = getProgram();

  let marketKeypair = anchor.web3.Keypair.generate();

  let authorizedKeypair = getAuthorizedUserKeypair();
  let feeManagerKeypair = getFeeManagerKeypair();
  let walletManager = getWalletManagerKeypair();
  let userKeypair = getUserKeypair();

  const marketPDA = getMarketPDA(marketKeypair.publicKey, program);

  const authorizedUserPda = getAuthorizedPDA(
    authorizedKeypair.publicKey,
    program
  );

  const userPDA = getUserPDA(userKeypair.publicKey, program);

  const marketPortfolioPDA = getMarketPortfolioPDA(marketPDA, userPDA, program);

  const from = getAssociatedTokenAddressSync(
    MINT,
    walletManager.publicKey
  );
  const feeAccount = getAssociatedTokenAddressSync(
    MINT,
    feeManagerKeypair.publicKey
  );

  let now = new Date();

  const initMarketData = [
    {
      id: new anchor.BN(1),
      initPot: new anchor.BN(100 * Math.pow(10,9)),
      choiceIds: [new anchor.BN(1), new anchor.BN(2)],
      fairLaunchStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 3),
      fairLaunchEnd: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 2),
      tradingStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() / 1000 + 60 * 60),
    },
    {
      id: new anchor.BN(2),
      initPot: new anchor.BN(100 * Math.pow(10,9)),
      choiceIds: [new anchor.BN(1), new anchor.BN(2)],
      fairLaunchStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 3),
      fairLaunchEnd: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 2),
      tradingStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() / 1000 + 60 * 60),
    }
  ]


  before(async () => {
    await program.methods
      .initMarket(initMarketData, marketKeypair.publicKey)
      .signers([authorizedKeypair])
      .accounts({
        signer: authorizedKeypair.publicKey,
        market: marketPDA,
        authorizedUser: authorizedUserPda,
      })
      .rpc()
      .catch((err) => console.log(err));

    const user = await program.account.user.fetch(userPDA);

    if (user.balance) {
      await program.methods
        .withdrawFromBalance(user.balance)
        .signers([walletManager])
        .accounts({
          user: userPDA,
          signer: walletManager.publicKey,
          mint: MINT,
          fromUsdcAccount: getWalletManagerTokenAccount(MINT),
          toUsdcAccount: await getAssociatedTokenAccount(
            MINT,
            userKeypair.publicKey
          ),
          feeUsdcAccount: feeAccount
        })
        .rpc();
    }

    await program.methods
      .addToBalance(new anchor.BN(5 * Math.pow(10,9)))
      .signers([walletManager])
      .accounts({
        user: userPDA,
        signer: walletManager.publicKey,
      })
      .rpc();
  });

  it("sucessfully created the market and reset user balance", async () => {
    const market = await program.account.market.fetch(marketPDA);
    const user = await program.account.user.fetch(userPDA);

    const subMarket = market.subMarkets[0];

    expect(user.balance.toNumber() / Math.pow(10,9)).to.equal(5)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current.toNumber() / Math.pow(10,9), 0)).to.equal(100);
  });

  it("buy bulk by price", async () => {
    await program.methods
      .bulkBuyByPrice([
        {
          amount: new anchor.BN(5 * Math.pow(10,9)),
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.51213,
        },
      ])
      .signers([walletManager])
      
      .accounts({
        signer: walletManager.publicKey,
        user: userPDA,
        market: marketPDA,
        marketPortfolio: marketPortfolioPDA,
        mint: MINT,
        fromUsdcAccount: from,
        feeUsdcAccount: feeAccount
      })
      .rpc().catch(err => console.log(err));
      
    const user = await program.account.user.fetch(userPDA);
    const market = await program.account.market.fetch(marketPDA);
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);

    expect(user.balance.toNumber() / Math.pow(10,9)).to.equal(0)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot.toNumber()).reduce((sum, current) => sum + current / Math.pow(10,9),0)).to.equal(104.975);
    expect(market.subMarkets[0].choices[0].potShares.toNumber() / Math.pow(10,9)).to.equal(95.260776375)
    expect(market.subMarkets[0].choices[1].potShares.toNumber() / Math.pow(10,9)).to.equal(104.975)
    expect(market.subMarkets[0].choices[0].usdcPot.toNumber() / Math.pow(10,9)).to.equal(54.975)
    expect(market.subMarkets[0].choices[0].mintedShares.toNumber() / Math.pow(10,9)).to.equal(9.714223625)
    expect(market.subMarkets[0].choices[0].fairLaunchPot.toNumber() / Math.pow(10,9)).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber() / Math.pow(10,9)).to.equal(9.714223625)
  })

  it("fails to claim winnings", async () => {
    try {
    await program.methods
        .claimWinnings(new anchor.BN(1), new anchor.BN(1))
        .signers([walletManager])
        .accounts({
        signer: walletManager.publicKey,
        user: userPDA,
        market: marketPDA,
        marketPortfolio: marketPortfolioPDA,
        })
        .rpc();
    } catch (err) {
        const error = err as anchor.AnchorError;
        let expectedMsg =
            "Market is not resolved yet, check back later.";
        expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  })

  it("resolves market", async () => {
    await program.methods
      .resolveMarket(new anchor.BN(1), new anchor.BN(1))
      .signers([walletManager])
      .accounts({
        signer: walletManager.publicKey,
        market: marketPDA,
        authorizedUser: authorizedUserPda,
        mint: MINT,
        fromUsdcAccount: from,
        feeUsdcAccount: feeAccount
      })
      .rpc().catch(err => console.log(err))

    const market = await program.account.market.fetch(marketPDA);
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot.toNumber()).reduce((sum, current) => sum + current / Math.pow(10,9),0)).to.equal(99.975);
    expect(market.subMarkets[0].resolved).to.equal(true)
    expect(market.subMarkets[0].choices[0].winningChoice).to.equal(true)
  })

  it("fails to claim due to not authorized", async () => {
    try {
        await program.methods
            .claimWinnings(new anchor.BN(1), new anchor.BN(1))
            .signers([userKeypair])
            .accounts({
            signer: userKeypair.publicKey,
            user: userPDA,
            market: marketPDA,
            marketPortfolio: marketPortfolioPDA,
            })
            .rpc();
    } catch (err) {
        const error = err as anchor.AnchorError;
        let expectedMsg =
            "You do not have the authorization to use this instruction."
        expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  })

  it("fails to claim the wrong choice", async () => {
    try {
        await program.methods
            .claimWinnings(new anchor.BN(1), new anchor.BN(2))
            .signers([walletManager])
            .accounts({
            signer: walletManager.publicKey,
            user: userPDA,
            market: marketPDA,
            marketPortfolio: marketPortfolioPDA,
            })
            .rpc();
    } catch (err) {
        const error = err as anchor.AnchorError;
        let expectedMsg =
            "This is not a winning choice.";
        expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  })

  it("it claims choice", async () => {
    await program.methods
        .claimWinnings(new anchor.BN(1), new anchor.BN(1))
        .signers([walletManager])
        .accounts({
        signer: walletManager.publicKey,
        user: userPDA,
        market: marketPDA,
        marketPortfolio: marketPortfolioPDA,
        })
        .rpc().catch(err => console.log(err));

    const user = await program.account.user.fetch(userPDA);
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);

    expect(user.balance.toNumber() / Math.pow(10,9)).to.equal(99.975)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber() / Math.pow(10,9)).to.equal(0)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].claimed).to.equal(true)
    
  })
});
