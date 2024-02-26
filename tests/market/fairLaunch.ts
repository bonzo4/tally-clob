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

describe("fair launch", () => {
  const MINT = new PublicKey("5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes");
  const program = getProgram();

  let marketKeypair = anchor.web3.Keypair.generate();

  let authorizedKeypair = getAuthorizedUserKeypair();
  let walletManagerKeypair = getWalletManagerKeypair();
  let feeManagerKeypair = getFeeManagerKeypair()
  let userKeypair = getUserKeypair();

  const marketPDA = getMarketPDA(marketKeypair.publicKey, program);

  const authorizedUserPda = getAuthorizedPDA(
    authorizedKeypair.publicKey,
    program
  );

  const userPDA = getUserPDA(userKeypair.publicKey, program);

  const from = getAssociatedTokenAddressSync(
    MINT,
    walletManagerKeypair.publicKey
  );
  const feeAccount = getAssociatedTokenAddressSync(
    MINT,
    feeManagerKeypair.publicKey
  );

  const marketPortfolioPDA = getMarketPortfolioPDA(
    marketPDA,
    userPDA,
    program
  );

  let now = new Date();

  const marketData = [
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
          price: 0.5,
        },
        {
          id: new anchor.BN(2),
          shares: new anchor.BN(0),
          totalPot: 0,
          winningChoice: false,
          price: 0.5,
        },
      ],
      fairLaunchStart: new anchor.BN((now.valueOf() / 1000) - 60 * 60),
      fairLaunchEnd: new anchor.BN((now.valueOf() / 1000) + 60 * 60),
      tradingStart: new anchor.BN((now.valueOf() / 1000) + 60 * 60),
      tradingEnd: new anchor.BN((now.valueOf() / 1000) + 60 * 60 * 2),
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
      .rpc().catch(err => console.log(err));

    const user = await program.account.user.fetch(userPDA);

    if (user.balance) {
      await program.methods
        .withdrawFromBalance(user.balance)
        .signers([walletManagerKeypair])
        .accounts({
          user: userPDA,
          signer: walletManagerKeypair.publicKey,
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
      .signers([walletManagerKeypair])
      .accounts({
        user: userPDA,
        signer: walletManagerKeypair.publicKey,
      })
      .rpc();
  });

  it("sucessfully created the market and reset user balance", async () => {
    const market = await program.account.market.fetch(marketPDA);
    const user = await program.account.user.fetch(userPDA);

    const subMarket = market.subMarkets[0];

    expect(subMarket.totalPot).to.equal(0);
    expect(user.balance).to.equal(10);
  })

  it("fails to buy by shares due to not clob manager", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          }
        ])
        .signers([userKeypair])
        .accounts({
          signer: userKeypair.publicKey,
          mint: MINT,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
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
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
          {
            amount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 0.5,
          }
        ])
        .signers([walletManagerKeypair])
        .accounts({
          signer: walletManagerKeypair.publicKey,
          user: userPDA,
          mint: MINT,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
          fromUsdcAccount: from,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Bulk order too big.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to no funds", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: 1000,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
        ])
        .signers([walletManagerKeypair])
        .accounts({
          signer: walletManagerKeypair.publicKey,
          user: userPDA,
          mint: MINT,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
          fromUsdcAccount: from,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Not enough balance to make order.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to estimation off", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            amount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.6,
          },
        ])
        .signers([walletManagerKeypair])
        .accounts({
          signer: walletManagerKeypair.publicKey,
          user: userPDA,
          mint: MINT,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
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
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
        ])
        .signers([walletManagerKeypair])
        .accounts({
          signer: walletManagerKeypair.publicKey,
          user: userPDA,
          mint: MINT,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
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
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
        ])
        .signers([walletManagerKeypair])
        .accounts({
          signer: walletManagerKeypair.publicKey,
          user: userPDA,
          mint: MINT,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
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

  it("buy bulk by shares", async () => {
    await program.methods
      .bulkBuyByShares([
        {
          amount: 5,
          subMarketId: new anchor.BN(2),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.5,
        },
      ])
      .signers([walletManagerKeypair])
      .accounts({
        signer: walletManagerKeypair.publicKey,
        user: userPDA,
        mint: MINT,
        market: marketPDA,
        marketPortfolio: marketPortfolioPDA,
        fromUsdcAccount: from,
        feeUsdcAccount: feeAccount
      })
      .rpc();

    const user = await program.account.user.fetch(userPDA);
    const market = await program.account.market.fetch(marketPDA);
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);


    expect(user.balance).to.equal(7.5)
    expect(market.subMarkets[0].totalPot).to.equal(2.5)
    expect(market.subMarkets[0].choices[0].totalPot).to.equal(2.5)
    expect(market.subMarkets[0].choices[0].shares.toNumber()).to.equal(5)
    expect(market.subMarkets[0].choices[0].price).to.equal(0.99)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares.toNumber()).to.equal(5)
  });

  it("buy bulk by price", async () => {
    await program.methods
      .bulkBuyByPrice([
        {
          amount: 2.5,
          subMarketId: new anchor.BN(2),
          choiceId: new anchor.BN(2),
          requestedPricePerShare: 0.5,
        },
      ])
      .signers([walletManagerKeypair])
      .accounts({
        signer: walletManagerKeypair.publicKey,
        user: userPDA,
        mint: MINT,
        market: marketPDA,
        marketPortfolio: marketPortfolioPDA,
        fromUsdcAccount: from,
        feeUsdcAccount: feeAccount
      })
      .rpc().catch(err => console.log(err));

    const user = await program.account.user.fetch(userPDA);
    const market = await program.account.market.fetch(marketPDA);
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);

    expect(user.balance).to.equal(5)
    expect(market.subMarkets[0].totalPot).to.equal(5)
    expect(market.subMarkets[0].choices[1].totalPot).to.equal(2.5)
    expect(market.subMarkets[0].choices[1].price).to.equal(0.5)
    expect(market.subMarkets[0].choices[1].shares.toNumber()).to.equal(5)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[1].shares.toNumber()).to.equal(5)
  });

  it("fails to sell by shares due to fair launch", async () => {
    try {
      await program.methods
        .bulkSellByShares([
          {
            amount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
        ])
        .signers([walletManagerKeypair])
        .accounts({
          signer: walletManagerKeypair.publicKey,
          user: userPDA,
          market: marketPDA,
          mint: MINT,
          marketPortfolio: marketPortfolioPDA,
          fromUsdcAccount: from,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Cannot sell at this time please check in when trading starts.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to sell by price due to fair launch", async () => {
    try {
      await program.methods
        .bulkSellByPrice([
          {
            amount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
        ])
        .signers([walletManagerKeypair])
        .accounts({
          signer: walletManagerKeypair.publicKey,
          user: userPDA,
          market: marketPDA,
          mint: MINT,
          marketPortfolio: marketPortfolioPDA,
          fromUsdcAccount: from,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Cannot sell at this time please check in when trading starts.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });
});
