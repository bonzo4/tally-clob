import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../../target/types/tally_clob";
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

  const marketData = [
    {
      id: new anchor.BN(1),
      totalPot: 0,
      choiceCount: 2,
      choices: [
        {
          id: new anchor.BN(1),
          shares: new anchor.BN(0),
          totalPot: 0,
          winningChoice: false,
          price: 1,
        },
        {
          id: new anchor.BN(2),
          shares: new anchor.BN(0),
          totalPot: 0,
          winningChoice: false,
          price: 1,
        },
      ],
      fairLaunchStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 3),
      fairLaunchEnd: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 2),
      tradingStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() / 1000 + 60 * 60),
      resolved: false,
    },
    {
      id: new anchor.BN(2),
      totalPot: 0,
      choiceCount: 2,
      choices: [
        {
          id: new anchor.BN(1),
          shares: new anchor.BN(0),
          totalPot: 0,
          winningChoice: false,
          price: 1,
        },
        {
          id: new anchor.BN(2),
          shares: new anchor.BN(0),
          totalPot: 0,
          winningChoice: false,
          price: 1,
        },
      ],
      fairLaunchStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 3),
      fairLaunchEnd: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 2),
      tradingStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() / 1000 + 60 * 60),
      resolved: false,
    },
  ];

  before(async () => {
    await program.methods
      .initMarket(marketData, marketKeypair.publicKey)
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
      .addToBalance(10)
      .signers([walletManager])
      .accounts({
        user: userPDA,
        signer: walletManager.publicKey,
      })
      .rpc();
  });

  // beforeEach(async() => {
  //   const user = await program.account.user.fetch(userPDA);
  //   console.log(user);
  //   const market = await program.account.market.fetch(marketPDA);
  //   console.log(market.subMarkets[0].totalPot, market.subMarkets[0].choices);
  //   const marketPortfolio = await program.account.marketPortfolio.fetch(
  //     marketPortfolioPDA
  //   ).then(portfolio => console.log(portfolio.subMarketPortfolio[0].choicePortfolio[0]))
  //   .catch(_ => {});
  
  // })

  // afterEach(async() => {
  //   const user = await program.account.user.fetch(userPDA);
  //   console.log(user);
  //   const market = await program.account.market.fetch(marketPDA);
  //   console.log(market.subMarkets[0].totalPot, market.subMarkets[0].choices);
  //   const marketPortfolio = await program.account.marketPortfolio.fetch(
  //     marketPortfolioPDA
  //   ).then(portfolio => console.log(portfolio.subMarketPortfolio[0].choicePortfolio[0]))
  //   .catch(_ => {});
  // })

  it("sucessfully created the market and reset user balance", async () => {
    const market = await program.account.market.fetch(marketPDA);
    const user = await program.account.user.fetch(userPDA);

    const subMarket = market.subMarkets[0];

    expect(subMarket.totalPot).to.equal(0);
    expect(user.balance).to.equal(10);
  });

  it("fails to buy by shares due to not clob manager", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 1,
          },
        ])
        .signers([userKeypair])
        .preInstructions([additionalComputeBudgetInstruction])
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
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 1,
          },
          {
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 1,
          },
          {
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 1,
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
      let expectedMsg = "Bulk order too big.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to same sub market", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 1,
          },
          {
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 1,
          }
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
      let expectedMsg = "Order cannot contain multiple multiples of the same sub market.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to no funds", async () => {
    try {
      
      await program.methods
        .bulkBuyByShares([
          {
            amount: 1200,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 1,
          },
          {
            amount: 1200,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 1,
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
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.94,
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
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 1,
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
        "The program expected this account to be already initialized";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to sell by shares due not having any shares", async () => {
    try {
      await program.methods
        .bulkSellByPrice([
          {
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 1,
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
        "The program expected this account to be already initialized";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("buy bulk by price", async () => {
    await program.methods
      .bulkBuyByPrice([
        {
          amount: 5,
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 1,
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

    const user = await program.account.user.fetch(userPDA);
    const market = await program.account.market.fetch(marketPDA);
    const marketPortfolio = await program.account.marketPortfolio.fetch(
      marketPortfolioPDA
    );

    expect(user.balance).to.greaterThan(5);
    expect(market.subMarkets[0].totalPot).to.lessThan(5);
    expect(market.subMarkets[0].choices[0].totalPot).to.lessThan(5);
    expect(market.subMarkets[0].choices[0].shares.toNumber()).to.equal(4);
    expect(market.subMarkets[0].choices[0].price).to.equal(1);
    expect(
      marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber()
    ).to.equal(4);
  });

  it("buy bulk by shares", async () => {
    await program.methods
      .bulkBuyByShares([
        {
          amount: 5,
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(2),
          requestedPricePerShare: 0.5,
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
      .rpc().catch(err => console.log(err));;

    const user = await program.account.user.fetch(userPDA);
    console.log(user)
    const market = await program.account.market.fetch(marketPDA);
    console.log(market.subMarkets[0].choices)
    const marketPortfolio = await program.account.marketPortfolio.fetch(
      marketPortfolioPDA
    );
    console.log(marketPortfolio.subMarketPortfolio[0].choicePortfolio)

    expect(user.balance).to.equal(7.5);
    expect(market.subMarkets[0].totalPot).to.equal(102.5);
    expect(market.subMarkets[0].choices[0].totalPot).to.equal(52.5);
    expect(market.subMarkets[0].choices[0].shares.toNumber()).to.equal(105);
    expect(market.subMarkets[0].choices[0].price).to.equal(0.55);
    expect(
      marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber()
    ).to.equal(5);
  });

  //   it("fails to sell by shares due to fair launch", async () => {
  //     try {
  //       await program.methods
  //         .bulkSellByShares([
  //           {
  //             amount: 1,
  //             subMarketId: new anchor.BN(1),
  //             choiceId: new anchor.BN(1),
  //             requestedPricePerShare: 0.5,
  //           },
  //         ])
  //         .signers([walletManager])
  //         .accounts({
  //           signer: walletManager.publicKey,
  //           user: userPDA,
  //           market: marketPDA,
  //           marketPortfolio: marketPortfolioPDA,
  //         })
  //         .rpc();
  //     } catch (err) {
  //       const error = err as anchor.AnchorError;
  //       let expectedMsg =
  //         "Cannot sell at this time please check in when trading starts.";
  //       expect(error.error.errorMessage).to.equal(expectedMsg);
  //     }
  //   });

  //   it("fails to sell by price due to fair launch", async () => {
  //     try {
  //       await program.methods
  //         .bulkSellByPrice([
  //           {
  //             amount: 1,
  //             subMarketId: new anchor.BN(1),
  //             choiceId: new anchor.BN(1),
  //             requestedPricePerShare: 0.5,
  //           },
  //         ])
  //         .signers([walletManager])
  //         .accounts({
  //           signer: walletManager.publicKey,
  //           user: userPDA,
  //           market: marketPDA,
  //           marketPortfolio: marketPortfolioPDA,
  //         })
  //         .rpc();
  //     } catch (err) {
  //       const error = err as anchor.AnchorError;
  //       let expectedMsg =
  //         "Cannot sell at this time please check in when trading starts.";
  //       expect(error.error.errorMessage).to.equal(expectedMsg);
  //     }
  //   });
});
