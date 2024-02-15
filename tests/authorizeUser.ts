import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../target/types/tally_clob";
import {  PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import { getAuthorizedUserKeypair, getOwnerKeypair } from "./utils/wallets";

describe("authorize user instruction", () => {

  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);
  const program = anchor.workspace.TallyClob as Program<TallyClob>;

  let userKeypair = getAuthorizedUserKeypair();
  let owner = getOwnerKeypair();

  const [authorizedUserPda, _] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("authorized_users"),
      userKeypair.publicKey.toBuffer(),
    ],
    program.programId
  );

  it ("unauthorized authorize fails", async () => {
    try {
        await program.methods
        .authorizeUser(true, userKeypair.publicKey)
        .signers([userKeypair])
        .accounts({signer: userKeypair.publicKey, authorizedUser: authorizedUserPda })
        .rpc()

    } catch (err) {
        const error = err as anchor.AnchorError;
        let expectedMsg =
            "You do not have the authorization to use this instruction.";
        expect(error.error.errorMessage).to.equal(expectedMsg);
    }
  })
  
  it ("authorizes a user", async () => {
    await program.methods.authorizeUser(true, userKeypair.publicKey)
        .signers([owner])
        .accounts({signer: owner.publicKey, authorizedUser: authorizedUserPda })
        .rpc()

        const user = await program.account.authorizedUser.fetch(authorizedUserPda);

        expect(user.authorized).to.equal(true);
  })

  it ("unauthorizes a user", async () => {
    await program.methods.authorizeUser(false, userKeypair.publicKey)
        .signers([owner])
        .accounts({signer: owner.publicKey, authorizedUser: authorizedUserPda })
        .rpc()

        const user = await program.account.authorizedUser.fetch(authorizedUserPda);

        expect(user.authorized).to.equal(false);
  })

  it ("authorizes a user again", async () => {
    await program.methods.authorizeUser(true, userKeypair.publicKey)
        .signers([owner])
        .accounts({signer: owner.publicKey, authorizedUser: authorizedUserPda })
        .rpc()

        const user = await program.account.authorizedUser.fetch(authorizedUserPda);

        expect(user.authorized).to.equal(true);
  })

});
