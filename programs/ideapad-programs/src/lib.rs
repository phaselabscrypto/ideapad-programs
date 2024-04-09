use std::{mem, str::FromStr};

use anchor_lang::prelude::*;

declare_id!("49KpHHeP9Hx2TBnHYLZvVYTpc1q2bt2NTvZdr4bMfFea");


mod error;
use anchor_spl::{associated_token::AssociatedToken, metadata::Metadata, token::{Mint, Token, TokenAccount}};
use error::IdeaPadErrorCode;
#[program]
pub mod ideapad_programs {
    use anchor_lang::solana_program::program::invoke_signed;
    use spl_stake_pool::state::Fee;

    use super::*;

    pub fn create_project<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateProject<'info>>,
        seed: Vec<u8>,
        redeemption_stamp: Option<i64>,
        min_stake_amount: u64,
    ) -> Result<()> {
        ctx.accounts.project.init(
            ctx.accounts.authority.key(),
            redeemption_stamp,
            min_stake_amount,
            seed,
            ctx.bumps.project,
        )?;

        let bump = ctx.bumps.project;

        let init_pool_ix = spl_stake_pool::instruction::initialize(
            &ctx.accounts.stake_pool_program.key(), 
            &ctx.accounts.stake_pool.key(), 
            &ctx.accounts.stake_pool_manager.key(), 
            &ctx.accounts.stake_pool_withdrawal_authority.key(), 
            &ctx.accounts.stake_pool_withdrawal_authority.key(), 
            &ctx.accounts.validator_list.key(), 
            &ctx.accounts.reserve_stake.key(), 
            &ctx.accounts.pool_mint.key(), 
            &ctx.accounts.project_fee_account.key(), 
            &ctx.accounts.token_program.key(), 
            None, 
            Fee {
                denominator: 1,
                numerator: 1
            }, 
            Fee {
                denominator: 0,
                numerator:0
            }, 
            Fee {
                denominator: 0,
                numerator: 0
            }, 
            0, 
            1
        );

        invoke_signed(
            &init_pool_ix, 
            &[
                ctx.accounts.stake_pool.to_account_info(),
                ctx.accounts.stake_pool_manager.to_account_info(),
                ctx.accounts.stake_pool_withdrawal_authority.to_account_info(),
                ctx.accounts.stake_pool_withdrawal_authority.to_account_info(),
                ctx.accounts.validator_list.to_account_info(),
                ctx.accounts.reserve_stake.to_account_info(),
                ctx.accounts.pool_mint.to_account_info(),
                ctx.accounts.project_fee_account.to_account_info(),
                ctx.accounts.token_program.to_account_info()
            ], 
            &[&[b"pool_manager".as_ref(), &ctx.accounts.project.key().to_bytes(), &[bump]]]
        )?;

        // let add_validator_ix = spl_stake_pool::instruction::add_

        Ok(())
    }

    pub fn create_contribution_reward<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateContributionReward<'info>>,
        reward_type: RewardType,
        cost: u64,
        quantity: Option<u32>,
        bump: u8,
    ) -> Result<()> {
        
        ctx.accounts.contribution_reward.init(reward_type, ctx.accounts.project.key(), ctx.accounts.reward_collection_mint.key(), cost, quantity, bump)?;

        Ok(())
    }


    pub fn deposit_sol<'info>(
        ctx: Context<DepositSol>
    ) -> Result<()>{

        // Blazesol for demo
        let stake_pool = ctx.accounts.stake_pool.key();
        let reserve_stake_account = ctx.accounts.reserve_stake_account.key();
        let manager_account = ctx.accounts.manager_account.key();
        let pool_mint = ctx.accounts.pool_mint.key();
        let stake_pool_withdrawal_authority = ctx.accounts.stake_pool_withdrawal_authority.key();

        let instruction = spl_stake_pool::instruction::deposit_sol(&spl_stake_pool::id(), &stake_pool, &stake_pool_withdrawal_authority, &reserve_stake_account, &ctx.accounts.payer.key(), &ctx.accounts.lst_token_account.key(), &manager_account, &ctx.accounts.lst_token_account.key(), &pool_mint, &ctx.accounts.token_program.key(), ctx.accounts.contribution_reward.cost);

        let account = [
            ctx.accounts.stake_pool.to_account_info(),
            ctx.accounts.stake_pool_withdrawal_authority.to_account_info(),
            ctx.accounts.reserve_stake_account.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.lst_token_account.to_account_info(),
            ctx.accounts.manager_account.to_account_info(),
            // ctx.accounts.
        ];
        

        Ok(())
    }


    pub fn change_state<'info>(
        ctx: Context<ChangeState>,
        state: ProjectState
    ) -> Result<()>{
        
        ctx.accounts.project.state = state;

        Ok(())
    }

    

}

#[derive(Accounts)]
#[instruction(
    seed: Vec<u8>
)]
pub struct CreateProject<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"project", seed.as_slice()],
        bump,
        space = Project::space(&seed)
    )]
    pub project: Account<'info, Project>,
    #[account(
        init, 
        payer = payer,
        seeds = [b"pool_manager", project.key().as_ref()],
        bump,
        space = PoolManager::space()
    )]
    pub stake_pool_manager : Account<'info, PoolManager>,

    #[account(
        // mint::authority = Pubkey::from_str("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1").unwrap()
    )]
    pub pool_mint: Account<'info, Mint>,

    pub stake_pool: AccountInfo<'info>,
    pub reserve_stake: AccountInfo<'info>,

    pub project_fee_account: Account<'info, TokenAccount>,

    /// Restrict this to a list we control
    pub validator_list: AccountInfo<'info>,

    pub reserve_stake_account: AccountInfo<'info>,
    pub manager_account: AccountInfo<'info>,
    pub stake_pool_withdrawal_authority : AccountInfo<'info>,

    pub stake_pool_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateContributionReward<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority
    )]
    pub project: Account<'info, Project>,

    #[account(
        init,
        payer = payer,
        seeds = [b"reward", project.key().as_ref(), &[project.contribution_reward_count]],
        bump,
        space = ContributionReward::space()
    )]
    pub contribution_reward: Account<'info, ContributionReward>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = project,
        mint::freeze_authority = project
    )]
    pub reward_collection_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = reward_collection_mint,
        associated_token::authority = authority
    )]
    pub reward_collection_token_account: Box<Account<'info, TokenAccount>>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
    pub token_metadata: Program<'info, Metadata>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority
    )]
    pub project: Account<'info, Project>,

    #[account(
        has_one = project,
        has_one = reward_collection_mint,
    )]
    pub contribution_reward: Account<'info, ContributionReward>,

    #[account(
        mint::authority = project,
        mint::freeze_authority = project
    )]
    pub reward_collection_mint: Account<'info, Mint>,

    #[account(
        associated_token::mint = reward_collection_mint,
        associated_token::authority = authority
    )]
    pub reward_collection_token_account: Box<Account<'info, TokenAccount>>,

    pub stake_vault: Account<'info, StakeVault>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = reward_collection_mint,
        associated_token::authority = stake_vault
    )]
    pub lst_token_account: Account<'info, TokenAccount>,

    #[account(
        mint::authority = Pubkey::from_str("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1").unwrap()
    )]
    pub pool_mint: Account<'info, Mint>,

    pub stake_pool: AccountInfo<'info>,

    pub reserve_stake_account: AccountInfo<'info>,
    pub manager_account: AccountInfo<'info>,
    pub stake_pool_withdrawal_authority : AccountInfo<'info>,
    
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
    pub token_metadata: Program<'info, Metadata>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ChangeState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub authority: Signer<'info>,

    #[account(
        has_one = authority,
        seeds = [b"project", project.seed.as_slice()],
        bump = project.bump
    )]
    pub project: Account<'info, Project>,

    pub system_program: Program<'info, System>,
}



#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum ProjectState {
    Draft,
    Raising,
    Funded,
    Complete,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ProjectConfig {
    redeemption_stamp: Option<i64>,
    min_stake_amount: u64,
}

#[account]
pub struct Project {
    pub state: ProjectState,
    pub authority: Pubkey,
    pub config: ProjectConfig,
    pub raising_at: Option<i64>,
    pub stake_pool: Pubkey,
    pub lst_mint: Pubkey,
    // Count for contribution_reward account pda generation
    pub contribution_reward_count: u8,
    pub seed: Vec<u8>,
    pub bump: u8,
}
/**
 * At the moment we use a random vec<u8> as identifier/seed for project, but this could be a parent collection nft
 * that each reward references creating a tree of association to the project and validating the tie to the project for a
 * reward/contrinution nft. ??
 */

impl Project {
    pub fn init(
        &mut self,
        authority: Pubkey,
        redeemption_stamp: Option<i64>,
        min_stake_amount: u64,
        seed: Vec<u8>,
        bump: u8,
    ) -> Result<()> {
        let config: ProjectConfig = ProjectConfig {
            redeemption_stamp,
            min_stake_amount,
        };
        self.authority = authority;
        self.config = config;
        self.state = ProjectState::Draft;
        self.bump = bump;
        self.raising_at = None;
        self.contribution_reward_count = 0;
        self.seed = seed;
        Ok(())
    }

    pub fn space(seed: &Vec<u8>) -> usize {
        8 + 1 + 32 + 32 + 32 + mem::size_of::<ProjectConfig>() + 1 + 1 + 4 + seed.len()
    }

    pub fn increment_contribution_reward_count(&mut self) -> Result<u8> {
        self.contribution_reward_count
            .checked_add(1)
            .ok_or(IdeaPadErrorCode::NumericalOverflow.into())
    }

}

// Nice in theory probably has to be fleshed out a bit more
#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum RewardType {
    Additive,  // Include all rewards up until this one that is additive
    Exclusive, // Only include this reward
}

#[account]
pub struct ContributionReward {
    pub reward_type: RewardType,
    pub project: Pubkey,
    // Nft used at the end to gain access to reward
    pub reward_collection_mint: Pubkey,
    pub cost: u64,
    pub quantity: Option<u32>, // if None it is unlimited
    pub bump: u8
}

impl ContributionReward {
    pub fn init(
        &mut self,
        reward_type: RewardType,
        project: Pubkey,
        reward_collection_mint: Pubkey,
        cost: u64,
        quantity: Option<u32>,
        bump: u8,
    ) -> Result<()> {
        self.reward_type = reward_type;
        self.project = project;
        self.reward_collection_mint = reward_collection_mint;
        self.cost = cost;
        self.quantity = quantity;
        self.bump = bump;

        Ok(())
    }

    pub fn space() -> usize {
        8 + 1 + 32 + 32 + mem::size_of::<u64>() + mem::size_of::<Option<u32>>() + 1
    }
}

#[account]
pub struct StakeVault {
    pub project: Pubkey,
    pub staker: Pubkey,
    pub is_claimed: bool,
    pub bump: u8
}

impl StakeVault {
    pub fn init(
        &mut self,
        project: Pubkey,
        staker: Pubkey,
        bump : i8
    ) -> Result<()> {
        self.project = project;
        self.staker = staker;
        Ok(())
    }

    pub fn space() -> usize {
        8 + 32 + 32 + 1
    }

}

#[account]
pub struct PoolManager {
    pub project: Pubkey,
    pub bump: u8
}

impl PoolManager {
    pub fn init(
        &mut self,
        project: Pubkey,
        staker: Pubkey,
        bump : i8
    ) -> Result<()> {
        self.project = project;
        Ok(())
    }

    pub fn space() -> usize {
        8 + 32 + 1
    }

}



