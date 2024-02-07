import { TallyClob } from "../../target/types/tally_clob";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { beforeEach } from "mocha";
import { Keypair, PublicKey } from "@solana/web3.js";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";

const { SystemProgram } = anchor.web3;

export let tallyClob: TallyProgram;

beforeEach(async () => {
    const provider = anchor.AnchorProvider.env()

    tallyClob = new TallyProgram(provider, anchor.workspace.TallyClob as Program<TallyClob>);
})

export class TallyProgram {
    readonly provider: anchor.AnchorProvider;
    readonly program: Program<TallyClob>;
    readonly operatorPk: PublicKey;
    readonly operatorWallet: NodeWallet;

    constructor(
        provider: anchor.AnchorProvider,
        program: Program<TallyClob>,
      ) {
        this.provider = provider;
        this.program = program;
        this.operatorPk = provider.wallet.publicKey;
        this.operatorWallet = provider.wallet as NodeWallet;
      }

    getRawProgram(): Program {
        return this.program as Program;
    }

    async fetchUserAccount(userPk: PublicKey) {
        return await this.program.account.user.fetch(userPk);
    }

    async fetchMarketAccount(marketPk: PublicKey) {
        return await this.program.account.market.fetch(marketPk);
    }

    async fetchMarketPortfolio(portfolioPk: PublicKey) {
        return await this.program.account.marketPortfolio.fetch(portfolioPk)
    }
}