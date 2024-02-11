import * as anchor from "@coral-xyz/anchor";
import owner from '../../owner.json'
import user from '../../user.json'
import authorizedUser from '../../authorized_user.json'
import walletManager from '../../wallet-manager.json'


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