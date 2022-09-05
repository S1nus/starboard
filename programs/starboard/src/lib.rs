use anchor_lang::prelude::*;
use anchor_lang::solana_program::msg;
use std::ops::DerefMut;

mod contexts;
use contexts::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const FEED_SEED: &[u8] = b"Feed";
const ROUND_SEED: &[u8] = b"Round";
const ESCROW_SEED: &[u8] = b"Escrow";

#[program]
pub mod starboard {
    use super::*;

    pub fn init_feed(
        ctx: Context<InitFeed>,
        id: [u8; 32],
        description: [u8; 32],
        update_interval: u32,
    ) -> Result<()> {
        let mut feed = ctx.accounts.feed.load_init()?;
        feed.description = description;
        feed.latest_finalized_value = 0;
        feed.latest_finalized_timestamp = 0;
        feed.update_interval = update_interval;
        feed.staking_round = None;
        feed.reporting_round = None;
        feed.committing_round = None;
        feed.certifying_round = None;
        feed.finalizing_round = None;
        feed.bump = *ctx.bumps.get("feed").unwrap();
        feed.started = false;
        feed.height = 0;
        Ok(())
    }

    pub fn init_round(
        ctx: Context<InitRound>,
        num: u8,
    ) -> Result<()> {
        let round_key = ctx.accounts.round.key().clone();
        let mut round = ctx.accounts.round.load_init()?;
        //round.current_stage = StageKind::Standby;
        round.current_stage = 0;
        round.staking_start_timestamp = 0;
        round.num_stakers = 0;
        round.reporting_start_timestamp = 0;
        round.committing_start_timestamp = 0;
        round.certifying_start_timestamp = 0;
        round.finalizing_start_timestamp = 0;
        round.bump = *ctx.bumps.get("round").unwrap();
        round.num = num;
        round.round_height = 0;
        let mut feed = ctx.accounts.feed.load_mut()?;
        match num {
            0 => {
                feed.staking_round = Some(round_key);
            },
            1 => {
                feed.reporting_round = Some(round_key);
            },
            2 => {
                feed.committing_round = Some(round_key);
            },
            3 => {
                feed.certifying_round = Some(round_key);
            },
            4 => {
                feed.finalizing_round = Some(round_key);
            },
            _ => {
                panic!("invalid round number");
            }
        }
        Ok(())
    }

    pub fn start_feed(ctx: Context<StartFeed>) -> Result<()> {
        let mut feed = ctx.accounts.feed.load_mut()?;
        require!(
            feed.staking_round.is_some() ||
            feed.reporting_round.is_some() ||
            feed.committing_round.is_some() ||
            feed.certifying_round.is_some() ||
            feed.finalizing_round.is_some(),
            StarboardError::NoRoundsError
        );
        feed.started = true;
        Ok(())
    }

    pub fn start_staking(
        ctx: Context<StartStaking>,
        num: u8
    ) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.slot;
        let round_key = ctx.accounts.round.key().clone();
        let mut feed = ctx.accounts.feed.load_mut()?;
        let mut round = ctx.accounts.round.load_mut()?;

        // 0 = standby
        require!(
            round.current_stage == 0,
            StarboardError::RoundNotReady
        );

        if feed.staking_round != Some(round_key) {
            if let Some(staking_round) = feed.staking_round {
                let old_round = AccountLoader::<Round>::try_from(&ctx.remaining_accounts[0])?;
                let old_round_data = old_round.load()?;
                let feed_not_staking = (old_round_data.staking_start_timestamp.checked_add(feed.update_interval.into()).unwrap()) < timestamp;
                require!(feed_not_staking, StarboardError::StakingInProgress);
            }
        }

        feed.staking_round = Some(round_key);
        round.current_stage = 1;
        round.staking_start_timestamp = timestamp;
        feed.height = feed.height.checked_add(1).unwrap();
        round.round_height = feed.height;
        msg!("Staking round started at {}", timestamp);

        Ok(())
    }

    pub fn stake(
        ctx: Context<Stake>,
        num: u8,
        round_height: u64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.slot;
        let mut escrow = ctx.accounts.escrow.load_init()?;
        let round_key = ctx.accounts.round.key().clone();
        let mut round = ctx.accounts.round.load_mut()?;
        escrow.round_height = round.round_height;
        escrow.staker = ctx.accounts.voter.key();
        escrow.feed = ctx.accounts.feed.key();
        escrow.bump = *ctx.bumps.get("escrow").unwrap();
        escrow.timestamp = timestamp;
        let mut feed = ctx.accounts.feed.load_mut()?;
        let round_is_staking = 
            feed.staking_round == Some(round_key) &&
            round.current_stage == 1 &&
            round.staking_start_timestamp.checked_add(feed.update_interval.into()).unwrap() > timestamp;
        require!(round_is_staking, StarboardError::RoundNotStaking);
        let mut voter_lamports = ctx.accounts.voter.try_borrow_mut_lamports()?;
        let escrow_info = &ctx.accounts.escrow.to_account_infos()[0];
        let mut escrow_lamports = escrow_info.try_borrow_mut_lamports()?;
        **voter_lamports = voter_lamports.checked_sub(feed.min_stake).unwrap();
        **escrow_lamports = voter_lamports.checked_add(feed.min_stake).unwrap();
        round.num_stakers = round.num_stakers.checked_add(1).unwrap();
        Ok(())
    }
}

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum StageKind {
    Standby,
    Staking,
    Reporting,
    Comitting,
    Certifying,
    Finalizing
}

#[account(zero_copy)]
pub struct Starboard {}

#[account(zero_copy)]
pub struct Round {
    pub current_stage: u8,

    pub staking_start_timestamp: u64,
    pub num_stakers: u32,
    pub reporting_start_timestamp: u64,
    pub committing_start_timestamp: u64,
    pub certifying_start_timestamp: u64,
    pub finalizing_start_timestamp: u64,
    pub bump: u8,
    pub num: u8,
    pub round_height: u64,
}

#[account(zero_copy)]
pub struct Feed {
    // string describing the feed, e.g "SOL/USD Spot Price"
    pub description: [u8; 32],
    // TokenAccount for feed's STARB holdings
    /// TODO
    //pub lease: Pubkey,
    pub latest_finalized_value: u64,
    pub latest_finalized_timestamp: u64,

    // how many slots the rounds last
    pub update_interval: u32,

    // Pubkeys of the Round PDAs for each of the five pipelined-stages
    pub staking_round: Option<Pubkey>,
    pub reporting_round: Option<Pubkey>,
    pub committing_round: Option<Pubkey>,
    pub certifying_round: Option<Pubkey>,
    pub finalizing_round: Option<Pubkey>,
    pub bump: u8,
    pub started: bool,
    pub height: u64,
    // in lamports
    pub min_stake: u64,
}

#[account(zero_copy)]
pub struct Escrow {
    pub round_height: u64,
    pub staker: Pubkey,
    pub feed: Pubkey,
    pub bump: u8,
    pub timestamp: u64,
}

#[error_code]
pub enum StarboardError {
    #[msg("No rounds initialized for the feed")]
    NoRoundsError,
    #[msg("Round not ready to change state")]
    RoundNotReady,
    #[msg("Staking for this feed already in progress")]
    StakingInProgress,
    #[msg("Cannot stake on this round")]
    RoundNotStaking,
}
