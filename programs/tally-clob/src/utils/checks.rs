use anchor_lang::prelude::*;

use crate::errors::TallyClobErrors;

pub fn is_authorized(signer_key: Pubkey) -> Result<()> {
    let payer = signer_key.to_string();
    let owner_address = String::from("FUqCpPJLmNLB7vVLp54ZCGadbQaeYVzZPENtSiL6VoVS");
    let _ = require!(payer == owner_address, TallyClobErrors::NotAuthorized);
    Ok(())
}