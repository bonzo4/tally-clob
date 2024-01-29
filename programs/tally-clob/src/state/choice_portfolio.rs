use anchor_lang::prelude::*;

use crate::{DISCRIMINATOR_SIZE, F64_SIZE, U8_SIZE};

#[account]
pub struct ChoicePortfolio {
    pub choice_index: usize,
    pub shares: f64
}

impl ChoicePortfolio {
    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U8_SIZE
    + F64_SIZE
    + F64_SIZE;

    pub fn new(
        choice_index: usize, 
        shares: f64
    ) -> ChoicePortfolio {
        ChoicePortfolio {
            choice_index,
            shares
        }
    } 
}