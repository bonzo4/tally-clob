import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../../target/types/tally_clob";
import {  PublicKey } from "@solana/web3.js";
import { expect } from "chai";

describe("init market", () => {

  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);
  const program = anchor.workspace.TallyClob as Program<TallyClob>;

  let market_id = anchor.web3.Keypair.generate()
  

  const [marketPDA, _] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("markets"),
      market_id.publicKey.toBuffer(),
    ],
    program.programId
  );

  let now = new Date();
  
  it ("creates a market", async () => {
    await program.methods.initMarket(
        [{
            id:new anchor.BN(1),
            totalPot: 0, 
            choiceCount: 2, 
            choices: [],
            fairLaunchStart: new anchor.BN(1704432733),
            fairLaunchEnd: new anchor.BN(1707111133),
            tradingStart: new anchor.BN(1709616733),
            tradingEnd: new anchor.BN(1712291533),
            resolved: false
        }],
        market_id.publicKey
    )
    .accounts({signer: provider.wallet.publicKey, market: marketPDA})
    .rpc()

    const market = await program.account.market.fetch(marketPDA)

    const subMarket = market.subMarkets[0];

    expect(subMarket.totalPot).to.equal(0);
  })

});
