use anchor_lang::prelude::*;

use crate::{vec_size, ChoicePortfolio, Market, DISCRIMINATOR_SIZE, U8_SIZE};

#[account]
pub struct MarketPortfolio {
    pub bump: u8,
    pub choice_count: u8,
    pub choice_portfolio: Vec<ChoicePortfolio>,
}

impl MarketPortfolio {

    pub const SIZE: usize = DISCRIMINATOR_SIZE
    + U8_SIZE
    + U8_SIZE
    + vec_size(ChoicePortfolio::SIZE, Market::CHOICE_MAX_LENGTH);

    pub fn add_to_portfolio(&mut self, choice_index: usize, shares: f64) -> Result<&Self> {
        match self
        .choice_portfolio
        .get_mut(choice_index) {
            Some (choice_portfolio) => {
                choice_portfolio.shares = shares
            }
            None => {
                self.choice_portfolio[choice_index] = ChoicePortfolio::new(choice_index, shares)
            }
        }

        Ok(self)
    }
}