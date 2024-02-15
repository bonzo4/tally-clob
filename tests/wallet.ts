import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../target/types/tally_clob";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { getUserKeypair, getWalletManagerKeypair } from "./utils/wallets";
import { before, beforeEach } from "mocha";
import {
  createAssociatedTokenAccount,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";

describe("wallet instructions", () => {
  const MINT = new PublicKey("5DUWZLh3zPKAAJKu7ftMJJrkBrKnq3zHPPmguzVkhSes");

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TallyClob as Program<TallyClob>;

  let userKeypair = getUserKeypair();
  let walletManagerKeypair = getWalletManagerKeypair();

  const [userWalletPDA, _] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("users"), userKeypair.publicKey.toBuffer()],
    program.programId
  );

  const from = getAssociatedTokenAddressSync(
    MINT,
    walletManagerKeypair.publicKey
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
      })
      .rpc()
      .catch((_) => {});
    }
    
  });

  it("creates a wallet", async () => {
    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(0);
  });

  it("adds to balance", async () => {
    // Add your test here.
    await program.methods
      .addToBalance(10)
      .signers([walletManagerKeypair])
      .accounts({ user: userWalletPDA, signer: walletManagerKeypair.publicKey })
      .rpc();

    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(10);
  });

  it("unauthorized add", async () => {
    // Add your test here.
    try {
      await program.methods
        .addToBalance(10)
        .signers([userKeypair])
        .accounts({ user: userWalletPDA, signer: userKeypair.publicKey })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "You do not have the authorization to use this instruction.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }

    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(10);
  });

  it("withdraws from balance", async () => {
    // Add your test here.
    await program.methods
      .withdrawFromBalance(5)
      .signers([walletManagerKeypair])
      .accounts({
        user: userWalletPDA,
        signer: walletManagerKeypair.publicKey,
        mint: MINT,
        fromUsdcAccount: from,
        toUsdcAccount: to,
      })
      .rpc();

    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(5);
  });

  it("fails to withdraw", async () => {
    try {
      await program.methods
        .withdrawFromBalance(10)
        .signers([walletManagerKeypair])
        .accounts({
          user: userWalletPDA,
          signer: walletManagerKeypair.publicKey,
          mint: MINT,
          fromUsdcAccount: from,
          toUsdcAccount: to,
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg = "Amount to withdraw can't be greater than balance.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }

    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(5);
  });

  it("unauthorized withdraw", async () => {
    // Add your test here.
    try {
      await program.methods
        .withdrawFromBalance(10)
        .signers([userKeypair])
        .accounts({
          user: userWalletPDA,
          signer: userKeypair.publicKey,
          mint: MINT,
          fromUsdcAccount: from,
          toUsdcAccount: to,
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "You do not have the authorization to use this instruction.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }

    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(5);
  });
});
