use std::ops::{AddAssign, SubAssign};

use anchor_lang::prelude::*;

use crate::{errors::TallyClobErrors, U128_SIZE};

use super::{DISCRIMINATOR_SIZE, U8_SIZE};

#[account]
pub struct User {
    pub bump: u8,
    pub balance: u128,
    pub unreedemable_balance: u128,
}

impl User {
    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U8_SIZE
    + (U128_SIZE * 2);

    pub fn add_to_balance(&mut self, amount: u128) -> Result<&Self> {
        require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

        self
            .balance
            .add_assign(amount);


        Ok(self)
    }

    pub fn add_to_unreedeemable(&mut self, amount: u128) -> Result<&Self> {
        require!(amount > 0, TallyClobErrors::AmountToAddTooLow);

        self.unreedemable_balance
            .add_assign(amount);

        Ok(self)
    }

    pub fn withdraw_real_balance(&mut self, amount: u128) -> Result<&Self> {
        require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.balance, TallyClobErrors::BalanceTooLow);

        self
            .balance
            .sub_assign(amount);
        
        Ok(self)
    }

    pub fn withdraw_from_balance(&mut self, amount: u128) -> Result<&Self>  {
        require!(amount > 0, TallyClobErrors::AmountToWithdrawTooLow);
        require!(amount <= self.balance + self.unreedemable_balance, TallyClobErrors::BalanceTooLow);

        let mut amount_to_withdraw = amount;
        if self.unreedemable_balance != 0 {
            if amount_to_withdraw <= self.unreedemable_balance {
                self.unreedemable_balance.sub_assign(amount_to_withdraw);
                return Ok(self)
            }
            amount_to_withdraw -= self.unreedemable_balance;
            self.unreedemable_balance = 0;
        }

        self
            .balance
            .sub_assign(amount_to_withdraw);
        
        Ok(self)
    }
}