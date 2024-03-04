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

  const initMarketData = [
    {
      id: new anchor.BN(1),
      choiceIds: [new anchor.BN(1), new anchor.BN(2)],
      fairLaunchStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 3),
      fairLaunchEnd: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 2),
      tradingStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() / 1000 + 60 * 60),
    },
    {
      id: new anchor.BN(2),
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
      .addToBalance(10)
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

    expect(subMarket.choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0)).to.equal(100);
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
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5 ,
          },
          {
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(2),
            requestedPricePerShare: 0.5,
          },
          {
            amount: 1,
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
            amount: 1,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.5,
          },
          {
            amount: 1,
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
            amount: 20,
            subMarketId: new anchor.BN(1),
            choiceId: new anchor.BN(1),
            requestedPricePerShare: 0.524,
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
            requestedPricePerShare: 0.1,
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
            amount: 1,
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
          amount: 5,
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
    console.log(user.balance)
    const market = await program.account.market.fetch(marketPDA);
    console.log(market.subMarkets[0].choices)
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);
    console.log(marketPortfolio.subMarketPortfolio[0].choicePortfolio)

    expect(user.balance).to.equal(5)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0) ).to.equal(104.975);
    expect(market.subMarkets[0].choices[0].potShares).to.equal(95.26077637532747)
    expect(market.subMarkets[0].choices[1].potShares).to.equal(104.975)
    expect(market.subMarkets[0].choices[0].usdcPot).to.equal(54.975)
    expect(market.subMarkets[0].choices[0].mintedShares).to.equal(9.714223624672528)
    expect(market.subMarkets[0].choices[0].fairLaunchPot).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares).to.equal(9.714223624672528)
  });

  it("buy bulk by price 2", async () => {
    await program.methods
      .bulkBuyByPrice([
        {
          amount: 1,
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
    console.log(user.balance)
    const market = await program.account.market.fetch(marketPDA);
    console.log(market.subMarkets[0].choices)
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);
    console.log(marketPortfolio.subMarketPortfolio[0].choicePortfolio)

    expect(user.balance).to.equal(0)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0) ).to.equal(109.95);
    expect(market.subMarkets[0].choices[0].potShares).to.equal(90.95043201455208)
    expect(market.subMarkets[0].choices[1].potShares).to.equal(109.94999999999999)
    expect(market.subMarkets[0].choices[0].usdcPot).to.equal(59.95)
    expect(market.subMarkets[0].choices[0].mintedShares).to.equal(18.99956798544791)
    expect(market.subMarkets[0].choices[0].fairLaunchPot).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares).to.equal(18.99956798544791)
  });

  // it("buy bulk by shares", async () => {
  //   await program.methods
  //     .bulkBuyByShares([
  //       {
  //         amount: 5,
  //         subMarketId: new anchor.BN(1),
  //         choiceId: new anchor.BN(1),
  //         requestedPricePerShare: 0.52,
  //       },
  //     ])
  //     .signers([walletManager])
      
  //     .accounts({
  //       signer: walletManager.publicKey,
  //       user: userPDA,
  //       market: marketPDA,
  //       marketPortfolio: marketPortfolioPDA,
  //       mint: MINT,
  //       fromUsdcAccount: from,
  //       feeUsdcAccount: feeAccount
  //     })
  //     .rpc().catch(err => console.log(err));
  

  //     const user = await program.account.user.fetch(userPDA);
  //     console.log(user.balance)
  //     const market = await program.account.market.fetch(marketPDA);
  //     console.log(market.subMarkets[0].choices)
  //     const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);
  //     console.log(marketPortfolio.subMarketPortfolio[0].choicePortfolio)
  
  //     expect(user.balance).to.equal(5)
  //     expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0) ).to.equal(104.975);
  //     expect(market.subMarkets[0].choices[0].potShares).to.equal(95.92543201455207)
  //     expect(market.subMarkets[0].choices[1].potShares).to.equal(104.975)
  //     expect(market.subMarkets[0].choices[0].usdcPot).to.equal(54.975)
  //     expect(market.subMarkets[0].choices[0].mintedShares).to.equal(9.049567985447922)
  //     expect(market.subMarkets[0].choices[0].fairLaunchPot).to.equal(50)
  //     expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares).to.equal(9.049567985447922)
  // });

  it("sells by shares", async () => {
    await program.methods
      .bulkSellByShares([
        {
          amount: 1.88945,
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.54332,
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
    console.log(user.balance)
    const market = await program.account.market.fetch(marketPDA);
    console.log(market.subMarkets[0].choices)
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);
    console.log(marketPortfolio.subMarketPortfolio[0].choicePortfolio)

    expect(user.balance).to.equal(0.5433219287066832)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0) ).to.equal(109.40121754939676);
    expect(market.subMarkets[0].choices[0].potShares).to.equal(91.40164956394885)
    expect(market.subMarkets[0].choices[1].potShares).to.equal(106.67435652059035)
    expect(market.subMarkets[0].choices[0].usdcPot).to.equal(59.40121754939677)
    expect(market.subMarkets[0].choices[0].mintedShares).to.equal(17.99956798544791)
    expect(market.subMarkets[0].choices[0].fairLaunchPot).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares).to.equal(17.99956798544791)
  });

  it("sells by shares 2", async () => {
    await program.methods
      .bulkSellByShares([
        {
          amount: 5,
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.5202,
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
    console.log(user.balance)
    const market = await program.account.market.fetch(marketPDA);
    console.log(market.subMarkets[0].choices)
    const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);
    console.log(marketPortfolio.subMarketPortfolio[0].choicePortfolio)

    expect(user.balance).to.equal(3.2202900575890783)
    expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0) ).to.equal(104.046998880157446);
    expect(market.subMarkets[0].choices[0].potShares).to.equal(96.08526840375981)
    expect(market.subMarkets[0].choices[1].potShares).to.equal(104.04699888015743)
    expect(market.subMarkets[0].choices[0].usdcPot).to.equal(54.046998880157446)
    expect(market.subMarkets[0].choices[0].mintedShares).to.equal(7.961730476397619)
    expect(market.subMarkets[0].choices[0].fairLaunchPot).to.equal(50)
    expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[0].shares).to.equal(7.961730476397619)
  });

  // it("sells by price", async () => {
  //   await program.methods
  //     .bulkSellByPrice([
  //       {
  //         amount: 1,
  //         subMarketId: new anchor.BN(1),
  //         choiceId: new anchor.BN(1),
  //         requestedPricePerShare: 0.5,
  //       },
  //     ])
  //     .signers([walletManager])
      
  //     .accounts({
  //       signer: walletManager.publicKey,
  //       user: userPDA,
  //       market: marketPDA,
  //       marketPortfolio: marketPortfolioPDA,
  //       mint: MINT,
  //       fromUsdcAccount: from,
  //       feeUsdcAccount: feeAccount
  //     })
  //     .rpc().catch(err => console.log(err));

  //     const user = await program.account.user.fetch(userPDA);
  //   console.log(user.balance)
  //   const market = await program.account.market.fetch(marketPDA);
  //   console.log(market.subMarkets[0].choices)
  //   const marketPortfolio = await program.account.marketPortfolio.fetch(marketPortfolioPDA);
  //   console.log(marketPortfolio.subMarketPortfolio[0].choicePortfolio)

  //   expect(user.balance).to.equal(5)
  //   expect(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => sum + current, 0)).to.equal(7);
  //   // expect(market.subMarkets[0].choices[1].totalPot).to.equal(2.5)
  //   // expect(market.subMarkets[0].choices[1].price).to.equal(0.5)
  //   // expect(market.subMarkets[0].choices[1].shares.toNumber()).to.equal(5)
  //   expect(marketPortfolio.subMarketPortfolio[0].choicePortfolio[1].shares).to.equal(5)
  // });

  it("sells by price large order, but fails", async () => {
    try {
    await program.methods
      .bulkSellByPrice([
        {
          amount: 30,
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
      console.log(err)
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "Requested shares to sell greater than owned shares.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  
});
