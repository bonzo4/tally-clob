import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../target/types/tally_clob";
import { PublicKey } from "@solana/web3.js";
import {
  getAssociatedTokenAccount,
  getAuthorizedPDA,
  getAuthorizedUserKeypair,
  getFeeManagerKeypair,
  getMarketPDA,
  getMarketPortfolioPDA,
  getOwnerKeypair,
  getProgram,
  getUserKeypair,
  getUserPDA,
  getWalletManagerKeypair,
  getWalletManagerTokenAccount,
} from "./utils";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";

(async () => {
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

  const from = getAssociatedTokenAddressSync(MINT, walletManager.publicKey);
  const feeAccount = getAssociatedTokenAddressSync(
    MINT,
    feeManagerKeypair.publicKey
  );

  let now = new Date();

  const marketData = [
    {
      id: new anchor.BN(1),
      choiceCount: 2,
      choiceIds: [new anchor.BN(1), new anchor.BN(2)],
      fairLaunchStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 3),
      fairLaunchEnd: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 2),
      tradingStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() / 1000 + 60 * 60),
      resolved: false,
    },
    {
      id: new anchor.BN(2),
      choiceCount: 2,
      choiceIds: [new anchor.BN(1), new anchor.BN(2)],
      fairLaunchStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 3),
      fairLaunchEnd: new anchor.BN(now.valueOf() / 1000 - 60 * 60 * 2),
      tradingStart: new anchor.BN(now.valueOf() / 1000 - 60 * 60),
      tradingEnd: new anchor.BN(now.valueOf() / 1000 + 60 * 60),
      resolved: false,
    },
  ];

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

  if (user.balance.toNumber()) {
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
        feeUsdcAccount: feeAccount,
      })
      .rpc();
  }

  await program.methods
    .addToBalance(new anchor.BN(10 * 1_000_000))
    .signers([walletManager])
    .accounts({
      user: userPDA,
      signer: walletManager.publicKey,
    })
    .rpc();

  await program.methods
    .bulkBuyByShares([
      {
        amount: new anchor.BN(40000),
        subMarketId: new anchor.BN(1),
        choiceId: new anchor.BN(1),
        requestedPricePerShare: new anchor.BN(1),
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
      feeUsdcAccount: feeAccount,
    })
    .rpc();

//   const user1 = await program.account.user.fetch(userPDA);
//   const market = await program.account.market.fetch(marketPDA);
//   const marketPortfolio = await program.account.marketPortfolio.fetch(
//     marketPortfolioPDA
//   );
})();
