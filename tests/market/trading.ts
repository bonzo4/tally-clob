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

describe("trading", () => {
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
        .rpc().catch((err) => console.log(err));
    }

    await program.methods
      .addToBalance(new anchor.BN(6 * Math.pow(10,9)))
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

    expect(subMarket.choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current.toNumber(), 0) / Math.pow(10,9)).to.equal(100);
    expect(user.balance.toNumber() / Math.pow(10,9)).to.equal(6);
  });

  it("fails to buy by shares due to not clob manager", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
        ])
        .signers([userKeypair])
        .accounts({
          signer: userKeypair.publicKey,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
          mint: MINT,
          fromUsdcAccount: from,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "You do not have the authorization to use this instruction.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to too many orders", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5 ,
          },
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 0.5,
          },
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 0.5,
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
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg = "Bulk order too big.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to same sub market", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 0.5,
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
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg = "Order cannot contain multiple multiples of the same sub market.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to no funds", async () => {
    try {
      
      await program.methods
        .bulkBuyByShares([
          {
            amount: new anchor.BN(20 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.524,
          },
        ])
        .signers([walletManager])
        .preInstructions([additionalComputeBudgetInstruction])
        .accounts({
          signer: walletManager.publicKey,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
          mint: MINT,
          fromUsdcAccount: from,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg = "Not enough balance to make order.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to estimation off", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.1,
          },
        ])
        .signers([walletManager])
        .preInstructions([additionalComputeBudgetInstruction])
        .accounts({
          signer: walletManager.publicKey,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
          mint: MINT,
          fromUsdcAccount: from,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Requested price is to far off from acutal price, cancelling order.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to sell by shares due not having any shares", async () => {
    try {
      await program.methods
        .bulkSellByShares([
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
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
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "The program expected this account to be already initialized";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to sell by shares due not having any shares", async () => {
    try {
      await program.methods
        .bulkSellByPrice([
          {
            amount: new anchor.BN(1 * Math.pow(10,9)),
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
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
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "The program expected this account to be already initialized";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("buy bulk by price", async () => {
    await program.methods
      .bulkBuyByPrice([
        {
          amount: new anchor.BN(5 * Math.pow(10,9)),
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.5121,
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
    
    expect(user.balance.toNumber() / Math.pow(10,9)).to.equal(1)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot.toNumber()).reduce((sum, current) => sum + current / Math.pow(10,9),0)).to.equal(104.975);
    expect(market.subMarkets[0].choices[0].potShares.toNumber() / Math.pow(10,9)).to.equal(95.260776375)
    expect(market.subMarkets[0].choices[1].potShares.toNumber() / Math.pow(10,9)).to.equal(104.975)
    expect(market.subMarkets[0].choices[0].usdcPot.toNumber() / Math.pow(10,9)).to.equal(54.975)
    expect(market.subMarkets[0].choices[0].mintedShares.toNumber() / Math.pow(10,9)).to.equal(9.714223625)
    expect(market.subMarkets[0].choices[0].fairLaunchPot.toNumber() / Math.pow(10,9)).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber() / Math.pow(10,9)).to.equal(9.714223625)
  });

  it("buy bulk by price 2", async () => {
    await program.methods
      .bulkBuyByPrice([
        {
          amount: new anchor.BN(1 * Math.pow(10,9)),
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.5433,
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
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot.toNumber()).reduce((sum, current) => sum + current / Math.pow(10,9),0)).to.equal(105.97);
    expect(market.subMarkets[0].choices[0].potShares.toNumber() / Math.pow(10,9)).to.equal(94.366330093)
    expect(market.subMarkets[0].choices[1].potShares.toNumber() / Math.pow(10,9)).to.equal(105.97)
    expect(market.subMarkets[0].choices[0].usdcPot.toNumber() / Math.pow(10,9)).to.equal(55.97)
    expect(market.subMarkets[0].choices[0].mintedShares.toNumber() / Math.pow(10,9)).to.equal(11.603669907)
    expect(market.subMarkets[0].choices[0].fairLaunchPot.toNumber() / Math.pow(10,9)).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber() / Math.pow(10,9)).to.equal(11.603669907)
  });


  it("sells by shares", async () => {
    await program.methods
      .bulkSellByShares([
        {
          amount: new anchor.BN(1.88945 * Math.pow(10,9)),
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.54332,
        },
      ])
      .signers([walletManager])
      .preInstructions([additionalComputeBudgetInstruction])
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

    expect(user.balance.toNumber() / Math.pow(10,9)).to.equal(0.99002694)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot.toNumber()).reduce((sum, current) => sum + current / Math.pow(10,9),0)).to.equal(104.974998051);
    expect(market.subMarkets[0].choices[0].potShares.toNumber() / Math.pow(10,9)).to.equal(95.260778144)
    expect(market.subMarkets[0].choices[1].potShares.toNumber() / Math.pow(10,9)).to.equal(104.974998051)
    expect(market.subMarkets[0].choices[0].usdcPot.toNumber() / Math.pow(10,9)).to.equal(54.974998051)
    expect(market.subMarkets[0].choices[0].mintedShares.toNumber() / Math.pow(10,9)).to.equal(9.714219907)
    expect(market.subMarkets[0].choices[0].fairLaunchPot.toNumber() / Math.pow(10,9)).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber() / Math.pow(10,9)).to.equal(9.714219907)
  });


  it("sells by price large order, but fails", async () => {
    try {
    await program.methods
      .bulkSellByPrice([
        {
          amount: new anchor.BN(30 * Math.pow(10,9)),
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.2279,
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
      .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Requested shares to sell greater than owned shares.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  
});
