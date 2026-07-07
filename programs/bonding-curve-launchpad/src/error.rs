use anchor_lang::prelude::*;

#[error_code]
pub enum BondingCurveLaunchpadError {
    #[msg("Invalid arguments")]
    InvalidArguments,
    #[msg("Overflow")]
    Overflow,
    #[msg("Underflow")]
    Underflow,
    #[msg("Supply cap exceeded")]
    SupplyCapExceeded,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
}
