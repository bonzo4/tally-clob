use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, F64_SIZE};

use super::{DISCRIMINATOR_SIZE, U8_SIZE};

#[account]
pub struct User {
    pub bump: u8,
    pub balance: f64,
}

impl User {
    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U8_SIZE
    + F64_SIZE;

    pub fn add_to_balance(&mut self, amount: f64) -> Result<&Self> {
        require!(amount > 0.0, TallyClobErrors::AmountToAddTooLow);

        self
            .balance
            .add_assign(amount);


        Ok(self)
    }

    pub fn withdraw_from_balance(&mut self, amount: f64) -> Result<&Self>  {
        require!(amount > 0.0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.balance, TallyClobErrors::BalanceTooLow);

        self
            .balance
            .sub_assign(amount);
        
        Ok(self)
    }
}