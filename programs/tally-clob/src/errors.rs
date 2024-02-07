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
    #[msg("Sub market not found.")]
    SubMarketNotFound,
    #[msg("Choice not found.")]
    ChoiceNotFound,
    #[msg("Choice portfolio not found, please buy some shares first.")]
    ChoicePortfolioNotFound,
    #[msg("Not enough balance to make order.")]
    BalanceTooLow,
    #[msg("Market is still intializing, please try again later.")]
    MarketIntializing,
    #[msg("Market is closed, please check the results.")]
    MarketClosed,
    #[msg("Market is not resolved yet, check back later.")]
    MarketNotResolved,
    #[msg("This is not a winning choice.")]
    NotWinningChoice,
    #[msg("Cannot sell at this time please check in when trading starts.")]
    NotSellingPeriod,
    #[msg("Cannot buy at this time please check in when its Fair Launch or Trading.")]
    NotBuyingPeriod,
    #[msg("Requested shares to sell greater than owned shares.")]
    NotEnoughSharesToSell,
    #[msg("Sub market portfolio not found, please buy some shares first.")]
    SubMarketPortfolioNotFound,
    #[msg("Bulk order too big")]
    BulkOrderTooBig,
    #[msg("You have already claimed this winnings")]
    AlreadyClaimed
}