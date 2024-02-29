import * as anchor from "@coral-xyz/anchor";
import owner from '../owner.json'
import user from '../user.json'
import authorizedUser from '../authorized_user.json'
import walletManager from '../wallet-manager.json'
import clobManager from '../clob-manager.json'
import feeManager from '../fee-manager.json'
import { createAssociatedTokenAccount, getAssociatedTokenAddress, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";
import { TallyClob } from "../target/types/tally_clob";

require('dotenv').config()

export function getProgram(): anchor.Program<TallyClob> {
    const provider = anchor.AnchorProvider.env()
    anchor.setProvider(new anchor.AnchorProvider(provider.connection, provider.wallet, {}));
    const program = anchor.workspace.TallyClob as anchor.Program<TallyClob>;
    return program
}

export function getOwnerKeypair(): anchor.web3.Keypair {
    return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(owner));
}

export function getUserKeypair(): anchor.web3.Keypair {
    return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(user));
}

export function getAuthorizedUserKeypair(): anchor.web3.Keypair {
    return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(authorizedUser));
}

export function getWalletManagerKeypair(): anchor.web3.Keypair {
    return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(walletManager));
}

export function getClobManagerKeypair():anchor.web3.Keypair {
    return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(clobManager));
}
export function getFeeManagerKeypair():anchor.web3.Keypair {
    return anchor.web3.Keypair.fromSecretKey(Uint8Array.from(feeManager));
}


export function getWalletManagerTokenAccount(mint: PublicKey): PublicKey {
    return getAssociatedTokenAddressSync(
        mint,
        getWalletManagerKeypair().publicKey
      );
}

export async function getAssociatedTokenAccount(mint: PublicKey, owner: PublicKey): Promise<PublicKey> {

    const provider = anchor.AnchorProvider.env();
    let account: PublicKey;

    try {
        account = await getAssociatedTokenAddress(mint, owner);
    } catch (_) {
        account = await createAssociatedTokenAccount(
            provider.connection,
            getWalletManagerKeypair(),
            mint,
            owner
        );
    }
    return account;
}

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