import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../target/types/tally_clob";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { getUserKeypair, getWalletManagerKeypair } from "./utils/getWallets";
import { beforeEach } from "mocha";

describe("wallet instructions", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TallyClob as Program<TallyClob>;

  let userKeypair = getUserKeypair();
  let walletManagerKeypair = getWalletManagerKeypair();

  const [userWalletPDA, _] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("users"), userKeypair.publicKey.toBuffer()],
    program.programId
  );

  beforeEach(async () => {
    await program.methods
      .initWallet(userKeypair.publicKey)
      .signers([walletManagerKeypair])
      .accounts({ user: userWalletPDA, signer: walletManagerKeypair.publicKey })
      .rpc()
      .catch(err => console.log(err));
  });

	it("creates a wallet", async () => {
		const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(0);
	})

  it("adds to balance", async () => {
    // Add your test here.
    await program.methods
      .addToBalance(10, userKeypair.publicKey)
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
        .addToBalance(10, userKeypair.publicKey)
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

    expect(user.balance).to.equal(0);
  });

  it("withdraws from balance", async () => {
    // Add your test here.
    await program.methods
      .addToBalance(10, userKeypair.publicKey)
      .signers([walletManagerKeypair])
      .accounts({ user: userWalletPDA, signer: walletManagerKeypair.publicKey })
      .rpc();

    // Add your test here.
    await program.methods
      .withdrawFromBalance(5, userKeypair.publicKey)
      .signers([walletManagerKeypair])
      .accounts({ user: userWalletPDA, signer: walletManagerKeypair.publicKey })
      .rpc();

    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(5);
  });

  it("fails to withdraw", async () => {
    try {
      await program.methods
        .withdrawFromBalance(10, userKeypair.publicKey)
        .signers([walletManagerKeypair])
        .accounts({
          user: userWalletPDA,
          signer: walletManagerKeypair.publicKey,
        })
        .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg = "Amount to withdraw can't be greater than balance.";
      expect(error.error.errorMessage).to.equal(expectedMsg);
    }

    const user = await program.account.user.fetch(userWalletPDA);

    expect(user.balance).to.equal(0);
  });

  it("unauthorized withdraw", async () => {
    // Add your test here.
    try {
      await program.methods
        .withdrawFromBalance(10, userKeypair.publicKey)
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

    expect(user.balance).to.equal(0);
  });
});
