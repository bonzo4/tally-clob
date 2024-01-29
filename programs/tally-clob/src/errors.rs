use anchor_lang::error_code;

#[error_code]
pub enum TallyClobErrors {
    #[msg("Amount to add can't be less than 0.")]
    AmountToAddTooLow,
    #[msg("Amount to withdraw can't be less than 0.")]
    AmountToWithdrawTooLow,
    #[msg("Amount to withdraw can't be greater than balance.")]
    AmountToWithdrawTooGreat,
    #[msg("You do not have the authorization to use this instruction.")]
    NotAuthorized,
    #[msg("Choice not found.")]
    ChoiceNotFound,
    #[msg("Choice portfolio not found, please buy some shares first.")]
    ChoicePortfolioNotFound,
    #[msg("Not enough balance to make order.")]
    BalanceTooLow,
    #[msg("Market is still intializing, please try again later.")]
    MarketIntializing,
    #[msg("Cannot sell at this time please check in when trading starts.")]
    NotSellingPeriod,
    #[msg("Cannot buy at this time please check in when its Fair Launch or Trading.")]
    NotBuyingPeriod
}