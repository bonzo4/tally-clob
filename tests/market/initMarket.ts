import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import {
  getAuthorizedUserKeypair,
  getOwnerKeypair,
  getUserKeypair,
} from "../utils/wallets";
import { getProgram } from "../utils/program";



describe("init market", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = getProgram();

  let market_id = anchor.web3.Keypair.generate();

  let authorizedKeypair = getAuthorizedUserKeypair();
  let user = getUserKeypair();

  const [marketPDA, _1] = PublicKey.findProgramAddressSync(
    [anchor.utils.bytes.utf8.encode("markets"), market_id.publicKey.toBuffer()],
    program.programId
  );

  const [authorizedUserPda, _2] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("authorized_users"),
      authorizedKeypair.publicKey.toBuffer(),
    ],
    program.programId
  );

  const [unauthorizedUserPda, _3] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("authorized_users"),
      user.publicKey.toBuffer(),
    ],
    program.programId
  );

  let now = new Date();

  const initMarketData = [
    {
      id: new anchor.BN(1),
      initPot: new anchor.BN(100 * Math.pow(10,9)),
      choiceIds: [new anchor.BN(1), new anchor.BN(2)],
      fairLaunchStart: new anchor.BN(now.valueOf()),
      fairLaunchEnd: new anchor.BN(now.valueOf()),
      tradingStart: new anchor.BN(now.valueOf()),
      tradingEnd: new anchor.BN(now.valueOf()),
    }
  ]


  it("creates a market", async () => {
    await program.methods
      .initMarket(
        initMarketData,
        market_id.publicKey
      )
      .signers([authorizedKeypair])
      .accounts({ 
        signer: authorizedKeypair.publicKey, 
        market: marketPDA,
        authorizedUser:  authorizedUserPda})
      .rpc();

    const market = await program.account.market.fetch(marketPDA);

    const subMarket = market.subMarkets[0];

    expect(Number(market.subMarkets[0].choices.map(choice => choice.usdcPot).reduce((sum, current) => BigInt(sum) + BigInt(current.toNumber()), BigInt(0)) / BigInt(Math.pow(10,9)))).to.equal(100);
  });

  it("unauthorized create", async () => {
    try {
      await program.methods
      .initMarket(
        initMarketData,
        market_id.publicKey
      )
      .signers([user])
      .accounts({ 
        signer: user.publicKey, 
        market: marketPDA,
        authorizedUser:  unauthorizedUserPda})
      .rpc();
    } catch (err) {
      const error = err as anchor.AnchorError;
      let expectedMsg =
        "The program expected this account to be already initialized";
      expect(error.error.errorMessage).to.equal(expectedMsg);
      
    }
  });
});
