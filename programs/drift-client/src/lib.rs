use crate::state::Config;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use borsh::{BorshDeserialize, BorshSerialize};
use clearing_house::context::{
    InitializeUserOptionalAccounts,
    ManagePositionOptionalAccounts as ClearingHouseManagePositionOptionalAccounts,
};
use clearing_house::controller::position::PositionDirection as ClearingHousePositionDirection;
use clearing_house::cpi::accounts::{
    ClosePosition as ClearingHouseClosePosition,
    DepositCollateral as ClearingHouseDepositCollateral, InitializeUserWithExplicitPayer,
    OpenPosition as ClearingHouseOpenPosition,
    WithdrawCollateral as ClearingHouseWithdrawCollateral,
};
use clearing_house::state::state::State;
use context::*;
use error::ErrorCode;

mod context;
mod error;
mod state;

declare_id!("EGovrRumVsvCzcvHSAYZxzsiUiMTTsMSRjuwUVSxYkXt");

#[program]
pub mod drift_client {
    use super::*;
    use crate::state::Config;

    pub fn initialize(
        ctx: Context<Initialize>,
        _config_nonce: u8,
        _collateral_vault_nonce: u8,
    ) -> ProgramResult {
        let collateral_account_key = ctx.accounts.collateral_vault.to_account_info().key;

        let (authority, authority_nonce) = Pubkey::find_program_address(
            &[ctx
                .accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref()],
            ctx.program_id,
        );

        if ctx.accounts.authority.key != &authority {
            return Err(ErrorCode::InvalidAuthority.into());
        }

        if ctx.accounts.collateral_vault.owner != authority {
            return Err(ErrorCode::InvalidCollateralAccountAuthority.into());
        }

        **ctx.accounts.config = Config {
            admin: ctx.accounts.admin.key(),
            collateral_vault: *collateral_account_key,
            authority,
            authority_nonce,
            clearing_house_user: Pubkey::default(),
            clearing_house_user_positions: Pubkey::default(),
        };

        Ok(())
    }

    pub fn initialize_user(ctx: Context<InitializeUser>, _user_nonce: u8) -> ProgramResult {
        let signature_seeds = [
            ctx.accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref(),
            bytemuck::bytes_of(&ctx.accounts.config.authority_nonce),
        ];
        let signers = &[&signature_seeds[..]];
        let cpi_program = ctx.accounts.clearing_house_program.to_account_info();
        let cpi_accounts = InitializeUserWithExplicitPayer {
            state: ctx.accounts.clearing_house_state.to_account_info(),
            user: ctx.accounts.clearing_house_user.to_account_info(),
            user_positions: ctx.accounts.clearing_house_user_positions.clone(),
            authority: ctx.accounts.authority.clone(),
            payer: ctx.accounts.admin.clone(),
            rent: ctx.accounts.rent.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
        clearing_house::cpi::initialize_user_with_explicit_payer(
            cpi_ctx,
            _user_nonce,
            InitializeUserOptionalAccounts {
                whitelist_token: false,
            },
        )?;

        let config = &mut ctx.accounts.config;

        config.clearing_house_user = ctx.accounts.clearing_house_user.key();
        config.clearing_house_user = ctx.accounts.clearing_house_user_positions.key();

        Ok(())
    }

    pub fn deposit_collateral(ctx: Context<DepositCollateral>, amount: u64) -> ProgramResult {
        // Send collateral to client collateral vault
        let signature_seeds = [
            ctx.accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref(),
            bytemuck::bytes_of(&ctx.accounts.config.authority_nonce),
        ];
        let signers = &[&signature_seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx
                .accounts
                .admin_collateral_account
                .to_account_info()
                .clone(),
            to: ctx.accounts.collateral_vault.to_account_info().clone(),
            authority: ctx.accounts.admin.clone(),
        };
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
        token::transfer(cpi_context, amount)?;

        // Send collateral from client collateral vault to clearing house
        let signature_seeds = [
            ctx.accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref(),
            bytemuck::bytes_of(&ctx.accounts.config.authority_nonce),
        ];
        let signers = &[&signature_seeds[..]];
        let cpi_program = ctx.accounts.clearing_house_program.to_account_info();
        let cpi_accounts = ClearingHouseDepositCollateral {
            state: ctx.accounts.clearing_house_state.to_account_info(),
            user: ctx.accounts.clearing_house_user.to_account_info(),
            user_positions: ctx.accounts.clearing_house_user_positions.to_account_info(),
            authority: ctx.accounts.authority.clone(),
            collateral_vault: ctx
                .accounts
                .clearing_house_collateral_vault
                .to_account_info(),
            user_collateral_account: ctx.accounts.collateral_vault.to_account_info(),
            markets: ctx.accounts.clearing_house_markets.to_account_info(),
            deposit_history: ctx
                .accounts
                .clearing_house_deposit_history
                .to_account_info(),
            funding_payment_history: ctx
                .accounts
                .clearing_house_funding_payment_history
                .to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
        clearing_house::cpi::deposit_collateral(cpi_ctx, amount)?;
        Ok(())
    }

    pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>, amount: u64) -> ProgramResult {
        // Withdraw collateral from clearing house to client vault
        let signature_seeds = [
            ctx.accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref(),
            bytemuck::bytes_of(&ctx.accounts.config.authority_nonce),
        ];
        let signers = &[&signature_seeds[..]];
        let cpi_program = ctx.accounts.clearing_house_program.to_account_info();
        let cpi_accounts = ClearingHouseWithdrawCollateral {
            state: ctx.accounts.clearing_house_state.to_account_info(),
            user: ctx.accounts.clearing_house_user.to_account_info(),
            user_positions: ctx.accounts.clearing_house_user_positions.to_account_info(),
            authority: ctx.accounts.authority.clone(),
            collateral_vault: ctx
                .accounts
                .clearing_house_collateral_vault
                .to_account_info(),
            collateral_vault_authority: ctx
                .accounts
                .clearing_house_collateral_vault_authority
                .to_account_info(),
            insurance_vault: ctx
                .accounts
                .clearing_house_insurance_vault
                .to_account_info(),
            insurance_vault_authority: ctx
                .accounts
                .clearing_house_insurance_vault_authority
                .to_account_info(),
            user_collateral_account: ctx.accounts.collateral_vault.to_account_info(),
            markets: ctx.accounts.clearing_house_markets.to_account_info(),
            deposit_history: ctx
                .accounts
                .clearing_house_deposit_history
                .to_account_info(),
            funding_payment_history: ctx
                .accounts
                .clearing_house_funding_payment_history
                .to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
        clearing_house::cpi::withdraw_collateral(cpi_ctx, amount)?;

        // Send collateral to client collateral vault
        let signature_seeds = [
            ctx.accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref(),
            bytemuck::bytes_of(&ctx.accounts.config.authority_nonce),
        ];
        let signers = &[&signature_seeds[..]];
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: ctx.accounts.collateral_vault.to_account_info().clone(),
            to: ctx
                .accounts
                .admin_collateral_account
                .to_account_info()
                .clone(),
            authority: ctx.accounts.authority.clone(),
        };
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
        token::transfer(cpi_context, amount)?;

        Ok(())
    }

    pub fn open_position<'info>(
        ctx: Context<'_, '_, '_, 'info, OpenPosition<'info>>,
        direction: PositionDirection,
        quote_asset_amount: u128,
        market_index: u64,
        limit_price: u128,
        optional_accounts: ManagePositionOptionalAccounts,
    ) -> ProgramResult {
        let signature_seeds = [
            ctx.accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref(),
            bytemuck::bytes_of(&ctx.accounts.config.authority_nonce),
        ];
        let signers = &[&signature_seeds[..]];
        let cpi_program: AccountInfo<'info> = ctx.accounts.clearing_house_program.to_account_info();
        let cpi_accounts = ClearingHouseOpenPosition {
            state: ctx.accounts.clearing_house_state.to_account_info(),
            user: ctx.accounts.clearing_house_user.to_account_info(),
            user_positions: ctx.accounts.clearing_house_user_positions.to_account_info(),
            authority: ctx.accounts.authority.clone(),
            markets: ctx.accounts.clearing_house_markets.to_account_info(),
            oracle: ctx.accounts.oracle.clone(),
            trade_history: ctx.accounts.clearing_house_trade_history.to_account_info(),
            funding_payment_history: ctx
                .accounts
                .clearing_house_funding_payment_history
                .to_account_info(),
            funding_rate_history: ctx
                .accounts
                .clearing_house_funding_rate_history
                .to_account_info(),
        };
        let remaining_accounts = ctx.remaining_accounts;
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers)
            .with_remaining_accounts(remaining_accounts.into());
        clearing_house::cpi::open_position(
            cpi_ctx,
            match direction {
                PositionDirection::Long => ClearingHousePositionDirection::Long,
                PositionDirection::Short => ClearingHousePositionDirection::Short,
            },
            quote_asset_amount,
            market_index,
            limit_price,
            ClearingHouseManagePositionOptionalAccounts {
                discount_token: optional_accounts.discount_token,
                referrer: optional_accounts.referrer,
            },
        )?;
        Ok(())
    }

    pub fn close_position<'info>(
        ctx: Context<'_, '_, '_, 'info, ClosePosition<'info>>,
        market_index: u64,
        optional_accounts: ManagePositionOptionalAccounts,
    ) -> ProgramResult {
        let signature_seeds = [
            ctx.accounts
                .clearing_house_state
                .to_account_info()
                .owner
                .as_ref(),
            bytemuck::bytes_of(&ctx.accounts.config.authority_nonce),
        ];
        let signers = &[&signature_seeds[..]];
        let cpi_program: AccountInfo<'info> = ctx.accounts.clearing_house_program.to_account_info();
        let cpi_accounts = ClearingHouseClosePosition {
            state: ctx.accounts.clearing_house_state.to_account_info(),
            user: ctx.accounts.clearing_house_user.to_account_info(),
            user_positions: ctx.accounts.clearing_house_user_positions.to_account_info(),
            authority: ctx.accounts.authority.clone(),
            markets: ctx.accounts.clearing_house_markets.to_account_info(),
            oracle: ctx.accounts.oracle.clone(),
            trade_history: ctx.accounts.clearing_house_trade_history.to_account_info(),
            funding_payment_history: ctx
                .accounts
                .clearing_house_funding_payment_history
                .to_account_info(),
            funding_rate_history: ctx
                .accounts
                .clearing_house_funding_rate_history
                .to_account_info(),
        };
        let remaining_accounts = ctx.remaining_accounts;
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers)
            .with_remaining_accounts(remaining_accounts.into());
        clearing_house::cpi::close_position(
            cpi_ctx,
            market_index,
            ClearingHouseManagePositionOptionalAccounts {
                discount_token: optional_accounts.discount_token,
                referrer: optional_accounts.referrer,
            },
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum PositionDirection {
    Long,
    Short,
}

impl Default for PositionDirection {
    // UpOnly
    fn default() -> Self {
        PositionDirection::Long
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ManagePositionOptionalAccounts {
    pub discount_token: bool,
    pub referrer: bool,
}
