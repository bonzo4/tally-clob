import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
// import { TallyClob } from "../../target/types/tally_clob";
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
} from "./utils/wallets";
import { getProgram } from "./utils/program";
import {
  getAuthorizedPDA,
  getMarketPDA,
  getMarketPortfolioPDA,
  getUserPDA,
} from "./utils/pdas";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { beforeEach } from "mocha";

describe("testing", () => {
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
      .addToBalance(100)
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
    expect(user.balance).to.equal(100);
  });

  it("scenario1", async() => {
    await program.methods
      .bulkBuyByPrice([
        {
          amount: 50,
          subMarketId: new anchor.BN(1),
          choiceId: new anchor.BN(1),
          requestedPricePerShare: 0.55,
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
  })

});
