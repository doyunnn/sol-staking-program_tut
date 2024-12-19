use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

use solana_program::clock::Clock;

declare_id!("ibZPAshT5FeuqaRTrbk8vbvT1jrdoM51oBmGFhqBrnh");

pub mod constants {
    pub const VAULT_SEED: &[u8] = b"vault";
    pub const STAKE_INFO_SEED: &[u8] = b"stake_info";
    pub const TOKEN_SEED: &[u8] = b"token";
}

#[program]
pub mod staking_program {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info_account;

        if stake_info.is_staked {
            return Err(ErrorCode::IsStaked.into());
        }

        if amount == 0 {
            return Err(ErrorCode::NoTokens.into());
        }

        let clock = Clock::get()?;

        stake_info.stake_at_slot = clock.slot;
        stake_info.is_staked = true;

        let stake_amount = (amount)
            .checked_mul(10u64.pow(ctx.accounts.mint.decimals as u32))
            .unwrap();

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.stake_account.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            stake_amount,
        )?;

        Ok(())
    }
    pub fn destake(ctx: Context<Destake>) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info_account;

        if !stake_info.is_staked {
            return Err(ErrorCode::NotStaked.into());
        }

        let clock = Clock::get()?;
        // let slots_passed = clock.slot - stake_info.stake_at_slot;

        let stake_amount = ctx.accounts.stake_account.amount;

        // 스테이킹 기간 계산 (슬롯 단위)
        let slots_staked = clock.slot.saturating_sub(stake_info.stake_at_slot);

        // 1분을 슬롯으로 변환 (1분 * 60초 * 1000ms / 400ms)
        // let thirty_minutes_in_slots = 30 * 60 * 1000 / 400;
        let thirty_minutes_in_slots = 1 * 60 * 1000 / 400;

        // 분당 1%의 보상률 계산
        let periods = slots_staked as f64 / thirty_minutes_in_slots as f64;
        let reward_rate = 0.001; // 0.1%

        // 보상 계산
        let reward = (stake_amount as f64 * reward_rate * periods) as u64;

        let bump = *ctx.bumps.get("token_vault_account").unwrap();
        let signer: &[&[&[u8]]] = &[&[constants::VAULT_SEED, &[bump]]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.token_vault_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.token_vault_account.to_account_info(),
                },
                signer,
            ),
            reward,
        )?;

        let staker = ctx.accounts.signer.key();
        let bump = *ctx.bumps.get("stake_account").unwrap();
        let signer: &[&[&[u8]]] = &[&[constants::TOKEN_SEED, staker.as_ref(), &[bump]]];

        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.stake_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.stake_account.to_account_info(),
                },
                signer,
            ),
            stake_amount,
        )?;

        stake_info.is_staked = false;
        stake_info.stake_at_slot = clock.slot;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [constants::VAULT_SEED],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = token_vault_account,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [constants::STAKE_INFO_SEED, signer.key().as_ref()],
        bump,
        payer = signer,
        space = 8 + std::mem::size_of::<StakeInfo>(),
    )]
    pub stake_info_account: Account<'info, StakeInfo>,

    #[account(
        init_if_needed,
        seeds = [constants::TOKEN_SEED, signer.key().as_ref()],
        bump,
        payer = signer,
        token::mint = mint,
        token::authority = stake_account,
    )]
    pub stake_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Destake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [constants::VAULT_SEED],
        bump,
    )]
    pub token_vault_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [constants::STAKE_INFO_SEED, signer.key().as_ref()],
        bump,
    )]
    pub stake_info_account: Account<'info, StakeInfo>,

    #[account(
        mut,
        seeds = [constants::TOKEN_SEED, signer.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct StakeInfo {
    pub stake_at_slot: u64,
    pub is_staked: bool,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Tokens are already staked")]
    IsStaked,
    #[msg("Tokens are not staked")]
    NotStaked,
    #[msg("No Tokens to stake")]
    NoTokens,
}
