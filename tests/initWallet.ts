import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../target/types/tally_clob";
import {  PublicKey } from "@solana/web3.js";
import { expect } from "chai";

describe("init wallet instruction", () => {

  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.TallyClob as Program<TallyClob>;

  const [userWalletPDA, _] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("users"),
      provider.wallet.publicKey.toBuffer(),
    ],
    program.programId
  );
  

  it("creates a wallet", async () => {
    
    // Add your test here.
    await program.methods
      .initWallet()
      .accounts({signer: provider.wallet.publicKey, user: userWalletPDA})
      .rpc()

    const user = await program.account.user.fetch(userWalletPDA)

    expect(user.balance).to.equal(0);
  });

  it("adds balance to wallet", async () => {
    await program.methods.addToBalance(10)
    .accounts({
      signer: provider.wallet.publicKey,
      user: userWalletPDA
    }).rpc()

    const user = await program.account.user.fetch(userWalletPDA)

    expect(user.balance).to.equal(10);
  });

  it("withdraws balance from wallet", async () => {
    await program.methods.withdrawFromBalance(5)
    .accounts({
      signer: provider.wallet.publicKey,
      user: userWalletPDA
    }).rpc()

    const user = await program.account.user.fetch(userWalletPDA)

    expect(user.balance).to.equal(5);
  });
});
