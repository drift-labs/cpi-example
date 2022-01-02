use anchor_lang::prelude::*;

use crate::state::Config;
use anchor_spl::token::{Mint, Token, TokenAccount};
use clearing_house::program::ClearingHouse;
use clearing_house::state::history::funding_rate::FundingRateHistory;
use clearing_house::state::history::trade::TradeHistory;
use clearing_house::state::{
    history::{deposit::DepositHistory, funding_payment::FundingPaymentHistory},
    market::Markets,
    state::State,
    user::{User, UserPositions},
};

#[derive(Accounts)]
#[instruction(
    config_nonce: u8,
    collateral_vault_nonce: u8,
)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        seeds = [b"drift_client".as_ref()],
        bump = config_nonce,
        payer = admin
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        constraint = &clearing_house_state.collateral_mint.eq(&collateral_mint.key())
    )]
    pub collateral_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        seeds = [b"collateral_vault".as_ref()],
        bump = collateral_vault_nonce,
        payer = admin,
        token::mint = collateral_mint,
        token::authority = authority
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    pub authority: AccountInfo<'info>,
    pub clearing_house_state: Box<Account<'info, State>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct InitializeUser<'info> {
    #[account(mut, signer)]
    pub admin: AccountInfo<'info>,
    #[account(
        mut,
        has_one = admin
    )]
    pub config: Box<Account<'info, Config>>,
    pub clearing_house_state: Box<Account<'info, State>>,
    #[account(mut)]
    pub clearing_house_user: AccountInfo<'info>,
    #[account(mut, signer)]
    pub clearing_house_user_positions: AccountInfo<'info>,
    #[account(
        constraint = &config.authority.eq(&authority.key())
    )]
    pub authority: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub clearing_house_program: Program<'info, ClearingHouse>,
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
    #[account(signer)]
    pub admin: AccountInfo<'info>,
    #[account(mut)]
    pub admin_collateral_account: Box<Account<'info, TokenAccount>>,
    #[account(
        has_one = admin
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        constraint = &config.collateral_vault.eq(&collateral_vault.key())
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = &config.authority.eq(&authority.key())
    )]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_state: Box<Account<'info, State>>,
    #[account(mut)]
    pub clearing_house_user: Box<Account<'info, User>>,
    #[account(mut)]
    pub clearing_house_collateral_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub clearing_house_markets: AccountLoader<'info, Markets>,
    #[account(mut)]
    pub clearing_house_user_positions: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_funding_payment_history: AccountLoader<'info, FundingPaymentHistory>,
    #[account(mut)]
    pub clearing_house_deposit_history: AccountLoader<'info, DepositHistory>,
    pub clearing_house_program: Program<'info, ClearingHouse>,
}

#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(signer)]
    pub admin: AccountInfo<'info>,
    #[account(mut)]
    pub admin_collateral_account: Box<Account<'info, TokenAccount>>,
    #[account(
        has_one = admin
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        mut,
        constraint = &config.collateral_vault.eq(&collateral_vault.key())
    )]
    pub collateral_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = &config.authority.eq(&authority.key())
    )]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_state: Box<Account<'info, State>>,
    #[account(mut)]
    pub clearing_house_user: Box<Account<'info, User>>,
    #[account(mut)]
    pub clearing_house_collateral_vault: Box<Account<'info, TokenAccount>>,
    pub clearing_house_collateral_vault_authority: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_insurance_vault: Box<Account<'info, TokenAccount>>,
    pub clearing_house_insurance_vault_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub clearing_house_markets: AccountLoader<'info, Markets>,
    #[account(mut)]
    pub clearing_house_user_positions: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_funding_payment_history: AccountLoader<'info, FundingPaymentHistory>,
    #[account(mut)]
    pub clearing_house_deposit_history: AccountLoader<'info, DepositHistory>,
    pub clearing_house_program: Program<'info, ClearingHouse>,
}

#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(signer)]
    pub admin: AccountInfo<'info>,
    #[account(
        has_one = admin
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
        constraint = &config.authority.eq(&authority.key())
    )]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_state: Box<Account<'info, State>>,
    #[account(mut)]
    pub clearing_house_user: Box<Account<'info, User>>,
    #[account(mut)]
    pub clearing_house_markets: AccountLoader<'info, Markets>,
    pub oracle: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_user_positions: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_funding_payment_history: AccountLoader<'info, FundingPaymentHistory>,
    #[account(mut)]
    pub clearing_house_funding_rate_history: AccountLoader<'info, FundingRateHistory>,
    #[account(mut)]
    pub clearing_house_trade_history: AccountLoader<'info, TradeHistory>,
    pub clearing_house_program: Program<'info, ClearingHouse>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(signer)]
    pub admin: AccountInfo<'info>,
    #[account(
    has_one = admin
    )]
    pub config: Box<Account<'info, Config>>,
    #[account(
    constraint = &config.authority.eq(&authority.key())
    )]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_state: Box<Account<'info, State>>,
    #[account(mut)]
    pub clearing_house_user: Box<Account<'info, User>>,
    #[account(mut)]
    pub clearing_house_markets: AccountLoader<'info, Markets>,
    pub oracle: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_user_positions: AccountInfo<'info>,
    #[account(mut)]
    pub clearing_house_funding_payment_history: AccountLoader<'info, FundingPaymentHistory>,
    #[account(mut)]
    pub clearing_house_funding_rate_history: AccountLoader<'info, FundingRateHistory>,
    #[account(mut)]
    pub clearing_house_trade_history: AccountLoader<'info, TradeHistory>,
    pub clearing_house_program: Program<'info, ClearingHouse>,
}
