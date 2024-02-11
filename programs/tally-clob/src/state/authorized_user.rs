use anchor_lang::prelude::*;

use crate::{BOOL_SIZE, DISCRIMINATOR_SIZE, U8_SIZE};

#[account]
pub struct AuthorizedUser {
    pub bump: u8,
    pub authorized: bool
}

impl AuthorizedUser {
    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U8_SIZE
    + BOOL_SIZE;

}