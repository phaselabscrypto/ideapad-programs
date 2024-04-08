use std::mem;

use anchor_lang::prelude::*;

declare_id!("49KpHHeP9Hx2TBnHYLZvVYTpc1q2bt2NTvZdr4bMfFea");
mod error;
use error::IdeaPadErrorCode;
#[program]
pub mod ideapad_programs {
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
        8 + 1 + 32 + mem::size_of::<ProjectConfig>() + 1 + 1 + 4 + seed.len()
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


