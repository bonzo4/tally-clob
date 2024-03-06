import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { getFeeManagerKeypair, getUserKeypair, getWalletManagerKeypair } from "./utils/wallets";
import { before, beforeEach } from "mocha";
import {
  createAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { getProgram } from "./utils/program";

describe("wallet instructions", () => {
  const MINT = new PublicKey("5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = getProgram();

  let userKeypair = getUserKeypair();
  let walletManagerKeypair = getWalletManagerKeypair();
  let feeManagerKeypair = getFeeManagerKeypair()

  const [userWalletPDA, _] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("users"), userKeypair.publicKey.toBuffer()],
    program.programId
  );

  const from = getAssociatedTokenAddressSync(
    MINT,
    walletManagerKeypair.publicKey
  );
  const feeAccount = getAssociatedTokenAddressSync(
    MINT,
    feeManagerKeypair.publicKey
  );

  let to: PublicKey;

  before(async () => {
    
    await createAssociatedTokenAccount(
      provider.connection,
      walletManagerKeypair,
      MINT,
      userKeypair.publicKey
    ).catch((_) => {});

    to = getAssociatedTokenAddressSync(MINT, userKeypair.publicKey);
    await program.methods
      .initWallet(userKeypair.publicKey)
      .signers([walletManagerKeypair])
      .accounts({ user: userWalletPDA, signer: walletManagerKeypair.publicKey })
      .rpc()
      .catch((_) => {});

    const user = await program.account.user.fetch(userWalletPDA);
    const balance = user.balance;
    if (balance) {
      await program.methods
      .withdrawFromBalance(balance)
      .signers([walletManagerKeypair])
      .accounts({
        user: userWalletPDA,
        signer: walletManagerKeypair.publicKey,
        mint: MINT,
        fromUsdcAccount: from,
        toUsdcAccount: to,
        feeUsdcAccount: feeAccount
      })
      .rpc()
      .catch((_) => {});
    }
    
  });

  it("creates a wallet", async () => {
    const user = await program.account.user.fetch(userWalletPDA);

    expect(Number(BigInt(user.balance.toNumber()))).to.equal(0);
  });

  it("adds to balance", async () => {
    // Add your test here.
    await program.methods
      .addToBalance(new anchor.BN(10 * Math.pow(10,9)))
      .signers([walletManagerKeypair])
      .accounts({ user: userWalletPDA, signer: walletManagerKeypair.publicKey })
      .rpc();

    const user = await program.account.user.fetch(userWalletPDA);

    expect(Number(BigInt(user.balance.toNumber()) / BigInt(Math.pow(10,9)))).to.equal(10);
  });

  it("unauthorized add", async () => {
    // Add your test here.
    try {
      await program.methods
        .addToBalance(new anchor.BN(10 * Math.pow(10,9)))
        .signers([userKeypair])
        .accounts({ user: userWalletPDA, signer: userKeypair.publicKey })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "You do not have the authorization to use this instruction.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("withdraws from balance", async () => {
    // Add your test here.
    await program.methods
      .withdrawFromBalance(new anchor.BN(5 * Math.pow(10,9)))
      .signers([walletManagerKeypair])
      .accounts({
        user: userWalletPDA,
        signer: walletManagerKeypair.publicKey,
        mint: MINT,
        fromUsdcAccount: from,
        toUsdcAccount: to,
        feeUsdcAccount: feeAccount
      })
      .rpc();

    const user = await program.account.user.fetch(userWalletPDA);

    expect(Number(BigInt(user.balance.toNumber()) / BigInt(Math.pow(10,9)))).to.equal(5);
  });

  it("fails to withdraw", async () => {
    try {
      await program.methods
        .withdrawFromBalance(new anchor.BN(10 * Math.pow(10,9)))
        .signers([walletManagerKeypair])
        .accounts({
          user: userWalletPDA,
          signer: walletManagerKeypair.publicKey,
          mint: MINT,
          fromUsdcAccount: from,
          toUsdcAccount: to,
          feeUsdcAccount: feeAccount
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg = "Amount to withdraw can't be greater than balance.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  });

  it("unauthorized withdraw", async () => {
    // Add your test here.
    try {
      await program.methods
        .withdrawFromBalance(new anchor.BN(10 * Math.pow(10,9)))
        .signers([userKeypair])
        .accounts({
          user: userWalletPDA,
          signer: userKeypair.publicKey,
          mint: MINT,
          fromUsdcAccount: from,
          toUsdcAccount: to,
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
});
