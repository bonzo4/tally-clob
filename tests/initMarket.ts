import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TallyClob } from "../target/types/tally_clob";
import {  PublicKey } from "@solana/web3.js";
import { expect } from "chai";

describe("init wallet instruction", () => {

  let market_id = "4fkYG52SSMfr6vHvwUoqCYwZFhuxH5K5s8z8Ys7r79Cc"

  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.TallyClob as Program<TallyClob>;

  const [marketPDA, _] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("markets"),
      new PublicKey(market_id).toBuffer(),
    ],
    program.programId
  );
  
  it ("creates a market", async () => {
    await program.methods.initMarket([{
        id:1, 
        title: "test", 
        totalPot: 0, 
        choiceCount: 2, 
        choices: [],
        fairLaunchStart: new anchor.BN(1704432733),
        fairLaunchEnd: new anchor.BN(1707111133),
        tradingStart: new anchor.BN(1709616733),
        tradingEnd: new anchor.BN(1712291533)}])
    .accounts({signer: provider.wallet.publicKey, market: marketPDA})
    .rpc()

    const market = await program.account.market.fetch(marketPDA)

    const subMarket = market.subMarkets[0];

    expect(subMarket.totalPot).to.equal(0);
  })

});
