use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, BOOL_SIZE, DISCRIMINATOR_SIZE, F64_SIZE, U64_SIZE};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct ChoicePortfolio {
    pub choice_id: u64,
    pub shares: u64,
    pub claimed: bool
}

impl ChoicePortfolio {
    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U64_SIZE
    + F64_SIZE
    + F64_SIZE
    + BOOL_SIZE;

    pub fn new(
        choice_id: u64
    ) -> ChoicePortfolio {
        ChoicePortfolio {
            choice_id,
            shares: 0,
            claimed: false
        }
    }

    pub fn add_to_portfolio(&mut self, shares: u64) -> Result<&Self> {
        require!(shares > 0, TallyClobErrors::AmountToAddTooLow);
        
        self.shares
            .add_assign(shares);

        Ok(self)
    }

    pub fn withdraw_from_portfolio(&mut self, shares: u64) -> Result<&Self> {
        require!(self.shares >= shares, TallyClobErrors::NotEnoughSharesToSell);

        self.shares
            .sub_assign(shares);
        
        Ok(self)
    }
}