import * as anchor from "@coral-xyz/anchor";
import owner from '../../owner.json'
import user from '../../user.json'
import authorizedUser from '../../authorized_user.json'
import walletManager from '../../wallet-manager.json'
import clobManager from '../../clob-manager.json'
import feeManager from '../../fee-manager.json'
import { createAssociatedTokenAccount, getAssociatedTokenAddress, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { PublicKey } from "@solana/web3.js";


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