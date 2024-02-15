import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../../target/types/tally_clob";
import { PublicKey } from "@solana/web3.js";
import { expect } from "chai";
import {
  getAuthorizedUserKeypair,
  getOwnerKeypair,
  getUserKeypair,
} from "../utils/wallets";



describe("init market", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.TallyClob as Program<TallyClob>;

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

  const marketData = [
    {
      id: new anchor.BN(1),
      totalPot: 0,
      choiceCount: 2,
      choices: [
        {
          id: new anchor.BN(1),
          shares: new anchor.BN(0),
          totalPot: 0,
          winningChoice: false,
          price: 0.5,
        },
        {
          id: new anchor.BN(2),
          shares: new anchor.BN(0),
          totalPot: 0,
          winningChoice: false,
          price: 0.5,
        },
      ],
      fairLaunchStart: new anchor.BN(now.valueOf()),
      fairLaunchEnd: new anchor.BN(now.valueOf()),
      tradingStart: new anchor.BN(now.valueOf()),
      tradingEnd: new anchor.BN(now.valueOf()),
      resolved: false,
    },
  ]

  it("creates a market", async () => {
    await program.methods
      .initMarket(
        marketData,
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

    expect(subMarket.totalPot).to.equal(0);
  });

  it("unauthorized create", async () => {
    try {
      await program.methods
      .initMarket(
        marketData,
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
