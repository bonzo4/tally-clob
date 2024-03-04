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

  const initMarketData = [
    {
      id: new anchor.BN(2),
      choiceIds: [new anchor.BN(1),new anchor.BN(2)],
      fairLaunchStart: new anchor.BN((now.valueOf() / 1000) - 60 * 60),
      fairLaunchEnd: new anchor.BN((now.valueOf() / 1000) + 60 * 60),
      tradingStart: new anchor.BN((now.valueOf() / 1000) + 60 * 60),
      tradingEnd: new anchor.BN((now.valueOf() / 1000) + 60 * 60 * 2),
    }
  ]


  before(async () => {

    await program.methods
      .initWallet(userKeypair.publicKey)
      .signers([walletManagerKeypair])
      .accounts({ user: userPDA, signer: walletManagerKeypair.publicKey })
      .rpc()
      .catch((_) => {});

    await program.methods
      .initMarket(initMarketData, marketKeypair.publicKey)
      .signers([authorizedKeypair])
      .accounts({
        signer: authorizedKeypair.publicKey,
        market: marketPDA,
        authorizedUser: authorizedUserPda,
      })
      .rpc()

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

    expect(subMarket.choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0)).to.equal(100);
    expect(user.balance).to.equal(10);
  })

  it("fails to fair launch order due to not clob manager", async () => {
    try {
      await program.methods
        .fairLaunchOrder([
          {
            amount: 5,
            subMarketId: new anchor.BN(2),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5 ,
          },
        ])
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
        "You do not have the authorization to use this instruction.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("fails to buy by shares due to no funds", async () => {
    try {
      await program.methods
        .fairLaunchOrder([
          {
            amount: 100,
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
          marketPortfolio: marketPortfolioPDA,
        })
        .rpc();
    } catch (err) {
      
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Not enough balance to make order.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it ("fair launch orders", async () => {
    await program.methods
      .fairLaunchOrder([
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
        market: marketPDA,
        marketPortfolio: marketPortfolioPDA,
      })
      .rpc();

    const user = await program.account.user.fetch(userPDA);
    const market = await program.account.market.fetch(marketPDA);
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);

    expect(user.balance).to.equal(5)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0) ).to.equal(105);
    expect(market.subMarkets[0].choices[0].potShares).to.equal(100.11357187078718)
    expect(market.subMarkets[0].choices[1].potShares).to.equal(110.12492905786593)
    expect(market.subMarkets[0].choices[0].usdcPot).to.equal(55)
    expect(market.subMarkets[0].choices[0].mintedShares).to.equal(5)
    expect(market.subMarkets[0].choices[0].fairLaunchPot).to.equal(55)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares).to.equal(5)
  })

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
            requestedPricePerShare: 0.5 ,
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
