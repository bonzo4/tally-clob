use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

pub fn is_wallet_manager(signer_key: Pubkey) -> Result<()> {
    let address = signer_key.to_string();
    let manager_address = String::from("7rTBUSkc8PHPW3VwGiPB4EbwHWxoSvVpMmbnAqRiGwWx");
    require!(address == manager_address, TallyClobErrors::NotAuthorized);
    Ok(())
}

pub fn is_owner(signer_key: Pubkey) -> Result<()> {
    let address = signer_key.to_string();
    let owner_address = String::from("JDXABA58QsRJnGX4EvNDbMWx76shqYrRi72t8cW3ow3P");
    require!(address == owner_address, TallyClobErrors::NotAuthorized);
    Ok(())
}

pub fn is_clob_manager(signer_key: Pubkey) -> Result<()> {
    let address = signer_key.to_string();
    let owner_address = String::from("FvX9MgvZuaASkJyoVispyK2R6eDdc5tgtJmjXiqPuMKC");
    require!(address == owner_address, TallyClobErrors::NotAuthorized);
    Ok(())
}