use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

pub fn is_authorized(signer_key: Pubkey) -> Result<()> {
    let payer = signer_key.to_string();
    let owner_address = String::from("JDXABA58QsRJnGX4EvNDbMWx76shqYrRi72t8cW3ow3P");
    let _ = require!(payer == owner_address, TallyClobErrors::NotAuthorized);
    Ok(())
}