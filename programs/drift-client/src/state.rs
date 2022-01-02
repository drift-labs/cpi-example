use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct Config {
    pub admin: Pubkey,
    pub collateral_vault: Pubkey,
    pub authority: Pubkey,
    pub authority_nonce: u8,
    pub clearing_house_user: Pubkey,
    pub clearing_house_user_positions: Pubkey,
}
