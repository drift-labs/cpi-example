use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Clearing house not collateral account authority")]
    InvalidCollateralAccountAuthority,
}
