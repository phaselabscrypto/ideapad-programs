use std::{mem, str::FromStr};

use anchor_lang::{prelude::*, anchor_lang::solana_program::stake::state::Stake};

declare_id!("49KpHHeP9Hx2TBnHYLZvVYTpc1q2bt2NTvZdr4bMfFea");

mod error;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{Metadata, MetadataAccount},
    token::{Mint, Token, TokenAccount},
};

use error::IdeaPadErrorCode;

#[program]
pub mod ideapad_programs {

    use anchor_lang::solana_program::program::{invoke, invoke_signed};
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
            ctx.accounts.stake_pool.key(),
            ctx.accounts.pool_mint.key(),
            seed,
            ctx.bumps.project,
        )?;

        let bump = ctx.bumps.project;

        let init_pool_ix = spl_stake_pool::instruction::initialize(
            &ctx.accounts.stake_pool_program.key(),
            &ctx.accounts.stake_pool.key(),
            &ctx.accounts.stake_pool_manager.key(),
            &ctx.accounts.stake_pool_manager.key(),
            &ctx.accounts.stake_pool_withdrawal_authority.key(),
            &ctx.accounts.validator_list.key(),
            &ctx.accounts.reserve_stake.key(),
            &ctx.accounts.pool_mint.key(),
            &ctx.accounts.project_fee_account.key(),
            &ctx.accounts.token_program.key(),
            None,
            Fee {
                denominator: 1,
                numerator: 1,
            },
            Fee {
                denominator: 0,
                numerator: 0,
            },
            Fee {
                denominator: 0,
                numerator: 0,
            },
            0,
            1,
        );

        invoke_signed(
            &init_pool_ix,
            &[
                ctx.accounts.stake_pool.to_account_info(),
                ctx.accounts.stake_pool_manager.to_account_info(),
                ctx.accounts.stake_pool_manager.to_account_info(),
                ctx.accounts
                    .stake_pool_withdrawal_authority
                    .to_account_info(),
                ctx.accounts.validator_list.to_account_info(),
                ctx.accounts.reserve_stake.to_account_info(),
                ctx.accounts.pool_mint.to_account_info(),
                ctx.accounts.project_fee_account.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
            &[&[
                b"pool_manager".as_ref(),
                &ctx.accounts.project.key().to_bytes(),
                &[bump],
            ]],
        )?;

        let add_validator_ix = spl_stake_pool::instruction::add_validator_to_pool(
            &ctx.accounts.stake_pool_program.key(),
            &ctx.accounts.stake_pool.key(),
            &ctx.accounts.stake_pool_withdrawal_authority.key(),
            &ctx.accounts.reserve_stake.key(),
            &ctx.accounts.stake_pool_withdrawal_authority.key(),
            &ctx.accounts.validator_list.key(),
            &ctx.accounts.stake_account.key(),
            &ctx.accounts.phase_validator.key(),
            None,
        );

        invoke_signed(
            &add_validator_ix,
            &[
                ctx.accounts.stake_pool.to_account_info(),
                ctx.accounts
                    .stake_pool_withdrawal_authority
                    .to_account_info(),
                ctx.accounts.reserve_stake.to_account_info(),
                ctx.accounts.validator_list.to_account_info(),
                ctx.accounts.stake_account.to_account_info(),
                ctx.accounts.phase_validator.to_account_info(),
                ctx.accounts.rent.to_account_info(),
                ctx.accounts.clock.to_account_info(),
                ctx.accounts.stake_history.to_account_info(),
                ctx.accounts.stake_config.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.stake_program.to_account_info(),
            ],
            &[&[
                b"pool_manager".as_ref(),
                &ctx.accounts.project.key().to_bytes(),
                &[bump],
            ]],
        )?;

        Ok(())
    }

    pub fn create_contribution_reward<'info>(
        ctx: Context<'_, '_, 'info, 'info, CreateContributionReward<'info>>,
        reward_type: RewardType,
        cost: u64,
        quantity: Option<u32>,
        bump: u8,
    ) -> Result<()> {
        ctx.accounts.contribution_reward.init(
            reward_type,
            ctx.accounts.project.key(),
            ctx.accounts.reward_collection_mint.key(),
            cost,
            quantity,
            bump,
        )?;

        Ok(())
    }

    // TODO add deposit authority so we can gate deposit through our program

    /*
        Deposits sol into validator, mints lst to program owned account. Mints Nft for redeeming amount to user.
        Lst yeild is sent to sent to the a projects token account not owned by the program.
     */
    pub fn deposit_sol<'info>(ctx: Context<DepositSol>) -> Result<()> {
        let stake_pool = ctx.accounts.stake_pool.key();
        let reserve_stake_account = ctx.accounts.reserve_stake_account.key();
        let manager_account = ctx.accounts.manager_account.key();
        let pool_mint = ctx.accounts.pool_mint.key();
        let stake_pool_withdrawal_authority = ctx.accounts.stake_pool_withdrawal_authority.key();

        let instruction = spl_stake_pool::instruction::deposit_sol(
            &spl_stake_pool::id(),
            &stake_pool,
            &stake_pool_withdrawal_authority,
            &reserve_stake_account,
            &ctx.accounts.payer.key(),
            &ctx.accounts.lst_token_account.key(),
            &manager_account,
            &ctx.accounts.lst_token_account.key(),
            &pool_mint,
            &ctx.accounts.token_program.key(),
            ctx.accounts.contribution_reward.cost,
        );

        let accounts = [
            ctx.accounts.stake_pool.to_account_info(),
            ctx.accounts
                .stake_pool_withdrawal_authority
                .to_account_info(),
            ctx.accounts.reserve_stake_account.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.lst_token_account.to_account_info(),
            ctx.accounts.project_fee_account.to_account_info(),
            ctx.accounts.project_fee_account.to_account_info(),
            ctx.accounts.pool_mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.token_program.to_account_info()
        ];

        invoke(&instruction, &accounts)?;

        let mint_to_context = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.nft_mint.to_account_info(),
                to: ctx.accounts.nft_token_account.to_account_info(),
                authority: ctx.accounts.warp.to_account_info(),
            },
            &signers,
        );
    
        anchor_spl::token::mint_to(mint_to_context, 1)?;
    
        let create_metadata_context = CpiContext::new_with_signer(
            ctx.accounts.token_metadata.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                mint_authority: ctx.accounts.warp.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.warp.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signers,
        );
    
        let uri = ctx
            .accounts
            .warp
            .nft_uri
            .clone()
            .unwrap_or(ctx.accounts.collection_metadata.data.uri.clone());
    
        anchor_spl::metadata::create_metadata_accounts_v3(
            create_metadata_context,
            DataV2 {
                name: ctx.accounts.collection_metadata.data.name.clone(),
                symbol: ctx.accounts.collection_metadata.data.symbol.clone(),
                uri,
                seller_fee_basis_points: ctx
                    .accounts
                    .collection_metadata
                    .data
                    .seller_fee_basis_points,
                creators: Some(vec![MetadataCreator::default().into()]),
                collection: None,
                uses: None,
            },
            true,
            true,
            None,
        )?;
    
        let create_master_edition_context = CpiContext::new_with_signer(
            ctx.accounts.token_metadata.to_account_info(),
            CreateMasterEditionV3 {
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                mint: ctx.accounts.nft_mint.to_account_info(),
                mint_authority: ctx.accounts.warp.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.warp.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
                edition: ctx.accounts.nft_master_edition.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
            &signers,
        );
    
        anchor_spl::metadata::create_master_edition_v3(create_master_edition_context, Some(0))?;
    
        let set_and_verify_context = CpiContext::new_with_signer(
            ctx.accounts.token_metadata.to_account_info(),
            SetAndVerifySizedCollectionItem {
                metadata: ctx.accounts.nft_metadata.to_account_info(),
                collection_authority: ctx.accounts.warp.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),
                update_authority: ctx.accounts.warp.to_account_info(),
                collection_mint: ctx.accounts.collection_mint.to_account_info(),
                collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
                collection_master_edition: ctx.accounts.collection_master_edition.to_account_info(),
            },
            &signers,
        );
    
        anchor_spl::metadata::set_and_verify_sized_collection_item(set_and_verify_context, None)?;

        Ok(())
    }

    pub fn change_state<'info>(ctx: Context<ChangeState>, state: ProjectState) -> Result<()> {
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
    pub stake_pool_manager: Account<'info, PoolManager>,

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
    pub stake_pool_withdrawal_authority: AccountInfo<'info>,

    pub stake_account: AccountInfo<'info>,
    pub phase_validator: AccountInfo<'info>,

    pub stake_pool_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub stake_history: Sysvar<'info, StakeHistory>,
    pub stake_config: AccountInfo<'info>,
    pub stake_program: AccountInfo<'info>,
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
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub wallet: Signer<'info>,

    #[account(
        mut
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
        init,
        payer = payer,
        associated_token::mint = reward_collection_mint,
        associated_token::authority = wallet
    )]
    pub reward_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = contribution_reward,
        mint::freeze_authority = contribution_reward
    )]
    pub nft_mint: Box<Account<'info, Mint>>,

    /// CHECK inside instruction
    #[account(
        mut,
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref()],
        bump,
        seeds::program = Metadata::id(),
        constraint = nft_metadata.collection.clone().unwrap().key == contribution_reward.reward_collection_mint,
        constraint = nft_metadata.collection.clone().unwrap().verified == true
    )]
    pub nft_metadata: Account<'info, MetadataAccount>,


    /// CHECK inside instruction
    #[account(
        mut,
        seeds = [b"metadata", Metadata::id().as_ref(), nft_mint.key().as_ref(), b"edition"],
        bump,
        seeds::program = Metadata::id()
    )]
    pub nft_master_edition: AccountInfo<'info>,

    pub stake_vault: Account<'info, StakeVault>,

    pub project_fee_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = pool_mint,
        associated_token::authority = stake_vault
    )]
    pub lst_token_account: Account<'info, TokenAccount>,

    #[account(
        address = project.lst_mint 
    )]
    pub pool_mint: Account<'info, Mint>,

    pub stake_pool: AccountInfo<'info>,

    pub reserve_stake_account: AccountInfo<'info>,
    pub manager_account: AccountInfo<'info>,
    pub stake_pool_withdrawal_authority: AccountInfo<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub token_program: Program<'info, Token>,
    pub token_metadata: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
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
        stake_pool: Pubkey,
        lst_mint: Pubkey,
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
        self.stake_pool = stake_pool;
        self.lst_mint = lst_mint;
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
    pub bump: u8,
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
    pub bump: u8,
}

impl StakeVault {
    pub fn init(&mut self, project: Pubkey, staker: Pubkey, bump: i8) -> Result<()> {
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
    pub bump: u8,
}

impl PoolManager {
    pub fn init(&mut self, project: Pubkey, staker: Pubkey, bump: i8) -> Result<()> {
        self.project = project;
        Ok(())
    }

    pub fn space() -> usize {
        8 + 32 + 1
    }
}
