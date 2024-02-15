import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../../target/types/tally_clob";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import {
  getAssociatedTokenAccount,
  getAuthorizedUserKeypair,
  getClobManagerKeypair,
  getOwnerKeypair,
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

describe("fair launch", () => {
  const MINT = new PublicKey("5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes");
  const program = getProgram();

  let marketKeypair = anchor.web3.Keypair.generate();

  let authorizedKeypair = getAuthorizedUserKeypair();
  let clobManger = getClobManagerKeypair();
  let walletManagerKeypair = getWalletManagerKeypair();
  let userKeypair = getUserKeypair();

  const marketPDA = getMarketPDA(marketKeypair.publicKey, program);

  const authorizedUserPda = getAuthorizedPDA(
    authorizedKeypair.publicKey,
    program
  );

  const userPDA = getUserPDA(userKeypair.publicKey, program);

  const marketPortfolioPDA = getMarketPortfolioPDA(
    marketKeypair.publicKey,
    userKeypair.publicKey,
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
      fairLaunchStart: new anchor.BN(now.valueOf() - 1000 * 60 * 5),
      fairLaunchEnd: new anchor.BN(now.valueOf() + 1000 * 60 * 5),
      tradingStart: new anchor.BN(now.valueOf() + 1000 * 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() + 1000 * 60 * 60),
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
      .rpc();

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

    const subMarket = market.subMarkets[0];

    expect(subMarket.totalPot).to.equal(0);
  })

  it("fails to buy by shares due to not clob manager", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            requestedAmount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            estimatedAmount: 1,
          }
        ], {marketKey: marketKeypair.publicKey, userKey: userKeypair.publicKey})
        .signers([userKeypair])
        .accounts({
          signer: userKeypair.publicKey,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
        })
        .rpc();
    } catch (err) {
      console.log(err)
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "You do not have the authorization to use this instruction.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to no funds", async () => {
    try {
      await program.methods
        .bulkBuyByShares([
          {
            requestedAmount: 1000,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            estimatedAmount: 1000,
          },
        ], {marketKey: marketKeypair.publicKey, userKey: userKeypair.publicKey})
        .signers([clobManger])
        .accounts({
          signer: clobManger.publicKey,
          user: userPDA,
          market: marketPDA,
          // marketPortfolio: marketPortfolioPDA,
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
            requestedAmount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            estimatedAmount: 2,
          },
        ], {marketKey: marketKeypair.publicKey, userKey: userKeypair.publicKey})
        .signers([clobManger])
        .accounts({
          signer: clobManger.publicKey,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Estimated shares is to far off from acutal shares, cancelling order";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to sell by shares due to fair launch", async () => {
    try {
      await program.methods
        .bulkSellByShares([
          {
            requestedAmount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            estimatedAmount: 2,
          },
        ])
        .signers([clobManger])
        .accounts({
          signer: clobManger.publicKey,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
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
            requestedAmount: 1,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            estimatedAmount: 2,
          },
        ])
        .signers([clobManger])
        .accounts({
          signer: clobManger.publicKey,
          user: userPDA,
          market: marketPDA,
          marketPortfolio: marketPortfolioPDA,
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Cannot sell at this time please check in when trading starts.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("buy by shares during fair launch", async () => {
    await program.methods
      .bulkBuyByShares([
        {
          requestedAmount: 5,
          subMarketId: new anchor.BN(2),
          choiceId: new anchor.BN(1),
          estimatedAmount: 5,
        },
      ], {marketKey: marketKeypair.publicKey, userKey: userKeypair.publicKey})
      .signers([clobManger])
      .accounts({
        signer: clobManger.publicKey,
        user: userPDA,
        market: marketPDA,
        marketPortfolio: marketPortfolioPDA,
      })
      .rpc();

    const user = await await program.account.user.fetch(userPDA);
    const market = await program.account.market.fetch(marketPDA);
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);
      
    expect(user.balance).to.equal(10 - (5 * 0.5))
    expect(market.subMarkets[0].totalPot).to.equal((5 * 0.5))
    expect(market.subMarkets[0].choices[0].totalPot).to.equal((5 * 0.5))
    expect(market.subMarkets[0].choices[0].shares).to.equal((5))
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares).to.equal((5))
  });
});
