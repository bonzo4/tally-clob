import { PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { TallyClob } from "../../target/types/tally_clob";

export function getUserPDA(userKey: PublicKey, program: anchor.Program<TallyClob>): PublicKey {
    const [userPDA, _] = PublicKey.findProgramAddressSync(
        [
          anchor.utils.bytes.utf8.encode("users"),
          userKey.toBuffer(),
        ],
        program.programId
      );

      return userPDA;
}

export function getMarketPDA(marketKey: PublicKey, program: anchor.Program<TallyClob>): PublicKey {
    const [userPDA, _] = PublicKey.findProgramAddressSync(
        [
          anchor.utils.bytes.utf8.encode("markets"),
          marketKey.toBuffer(),
        ],
        program.programId
      );

      return userPDA;
}

export function getAuthorizedPDA(userKey: PublicKey, program: anchor.Program<TallyClob>): PublicKey {
    const [userPDA, _] = PublicKey.findProgramAddressSync(
        [
          anchor.utils.bytes.utf8.encode("authorized_users"),
          userKey.toBuffer(),
        ],
        program.programId
      );

      return userPDA;
}

export function getMarketPortfolioPDA(
    marketKey: PublicKey, 
    userKey: PublicKey, 
    program: anchor.Program<TallyClob>
): PublicKey {
    const [marketPortfolioPDA, _] = PublicKey.findProgramAddressSync(
        [
            anchor.utils.bytes.utf8.encode("market_portfolios"),
            marketKey.toBuffer(),
            userKey.toBuffer(),
        ],
        program.programId
      );

      return marketPortfolioPDA;
}