use anchor_lang::prelude::*;
use anchor_lang::solana_program::msg;
use std::ops::DerefMut;
use solana_program::pubkey;
use anchor_spl::{
    mint, 
    token::{
        TokenAccount, 
        Mint, 
        Token,
        Transfer,
        transfer
    },
    associated_token::{
        AssociatedToken
    },
};

mod contexts;
use contexts::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const FEED_SEED: &[u8] = b"Feed";
const ROUND_SEED: &[u8] = b"Round";
const ESCROW_SEED: &[u8] = b"Escrow";
const ESCROW_TOKEN_SEED: &[u8] = b"EscrowToken";
const REPORT_RECORD_SEED: &[u8] = b"ReportRecordSeed";
const CERT_RECORD_SEED: &[u8] = b"CertRecordSeed";
const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

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
        feed.min_stake = 5;
        Ok(())
    }

    pub fn init_round(
        ctx: Context<InitRound>,
        num: u8,
    ) -> Result<()> {
        let round_key = ctx.accounts.round.key().clone();
        let mut round = ctx.accounts.round.load_init()?;
        //round.current_stage = StageKind::Standby;
        round.feed = ctx.accounts.feed.key().clone();
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
            if let Some(_staking_round) = feed.staking_round {
                let old_round = AccountLoader::<Round>::try_from(&ctx.remaining_accounts[0])?;
                let old_round_data = old_round.load()?;
                let feed_not_staking = (old_round_data.staking_start_timestamp.checked_add(feed.update_interval.into()).unwrap()) < timestamp;
                require!(feed_not_staking, StarboardError::StakingInProgress);
            }
        }

        feed.staking_round = Some(round_key);
        round.current_stage = 1;
        round.staking_start_timestamp = timestamp;
        msg!("feed height: {}", feed.height);
        feed.height = feed.height.checked_add(1u64).unwrap();
        round.round_height = feed.height;
        msg!("feed height: {}", feed.height);
        msg!("round height: {}", round.round_height);
        msg!("Staking round started at {}", timestamp);

        Ok(())
    }

    pub fn stake(
        ctx: Context<Stake>,
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
        let feed = ctx.accounts.feed.load_mut()?;
        let round_is_staking = 
            feed.staking_round == Some(round_key) &&
            round.current_stage == 1 &&
            round.staking_start_timestamp.checked_add(feed.update_interval.into()).unwrap() > timestamp;
        require!(round_is_staking, StarboardError::RoundNotStaking);
        round.num_stakers = round.num_stakers.checked_add(1).unwrap();
        let transfer_accounts = Transfer {
            from: ctx.accounts.voter_token_account.to_account_info(),
            to: ctx.accounts.escrow_token.to_account_info(),
            authority: ctx.accounts.voter.to_account_info(),
        };
        let cpi_context = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts);
        transfer(cpi_context, feed.min_stake)?;
        Ok(())
    }

    pub fn start_reporting(ctx: Context<StartReporting>) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.slot;
        let round_key = ctx.accounts.round.key().clone();
        let mut feed = ctx.accounts.feed.load_mut()?;
        let mut round = ctx.accounts.round.load_mut()?;

        // 1 = staking
        require!(
            round.current_stage == 1,
            StarboardError::RoundNotReady
        );
        if feed.reporting_round != Some(round_key) {
            if let Some(_reporting_round) = feed.reporting_round {
                let old_round = AccountLoader::<Round>::try_from(&ctx.remaining_accounts[0])?;
                let old_round_data = old_round.load()?;
                let feed_not_reporting = (old_round_data.reporting_start_timestamp.checked_add(feed.update_interval.into()).unwrap()) < timestamp;
                require!(feed_not_reporting, StarboardError::ReportingInProgress);
            }
        };

        feed.reporting_round = Some(round_key);
        round.current_stage = 2;
        round.reporting_start_timestamp = timestamp;
        Ok(())

    }

    pub fn report(
        ctx: Context<Report>,
        val: u64,
        confidence_interval: u8
    ) -> Result<()> {
        /// TODO
        Ok(())
    }

    pub fn start_committing(ctx: Context<StartComitting>) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.slot;
        let round_key = ctx.accounts.round.key().clone();
        let mut feed = ctx.accounts.feed.load_mut()?;
        let mut round = ctx.accounts.round.load_mut()?;

        // 2 = reporting
        require!(
            round.current_stage == 2,
            StarboardError::RoundNotReady
        );
        if feed.committing_round != Some(round_key) {
            if let Some(_committing_round) = feed.committing_round {
                let old_round = AccountLoader::<Round>::try_from(&ctx.remaining_accounts[0])?;
                let old_round_data = old_round.load()?;
                let feed_not_committing = (old_round_data.committing_start_timestamp.checked_add(feed.update_interval.into()).unwrap()) < timestamp;
                require!(feed_not_committing, StarboardError::ComittingInProgress);
            }
        };

        feed.committing_round = Some(round_key);
        round.current_stage = 3;
        round.committing_start_timestamp = timestamp;
        Ok(())

    }

    pub fn committ(
        ctx: Context<Comitt>,
        commitment: [u8; 32],
    ) -> Result<()> {
        Ok(())
    }

    pub fn start_certifying(ctx: Context<StartCertifying>) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.slot;
        let round_key = ctx.accounts.round.key().clone();
        let mut feed = ctx.accounts.feed.load_mut()?;
        let mut round = ctx.accounts.round.load_mut()?;

        // 3 = committing
        require!(
            round.current_stage == 3,
            StarboardError::RoundNotReady
        );
        if feed.certifying_round != Some(round_key) {
            if let Some(_certifying_round) = feed.certifying_round {
                let old_round = AccountLoader::<Round>::try_from(&ctx.remaining_accounts[0])?;
                let old_round_data = old_round.load()?;
                let feed_not_certifying = (old_round_data.certifying_start_timestamp.checked_add(feed.update_interval.into()).unwrap()) < timestamp;
                require!(feed_not_certifying, StarboardError::CertifyingInProgress);
            }
        };

        feed.certifying_round = Some(round_key);
        round.current_stage = 4;
        round.certifying_start_timestamp = timestamp;
        Ok(())
    }

    pub fn certify(ctx: Context<Certify>) -> Result<()> {
        Ok(())
    }

    pub fn start_finalizing(ctx: Context<StartFinalizing>) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp = clock.slot;
        let round_key = ctx.accounts.round.key().clone();
        let mut feed = ctx.accounts.feed.load_mut()?;
        let mut round = ctx.accounts.round.load_mut()?;

        // 4 = certifying
        require!(
            round.current_stage == 4,
            StarboardError::RoundNotReady
        );
        if feed.finalizing_round != Some(round_key) {
            if let Some(_finalizing_round) = feed.finalizing_round {
                let old_round = AccountLoader::<Round>::try_from(&ctx.remaining_accounts[0])?;
                let old_round_data = old_round.load()?;
                let feed_not_finalizing = (old_round_data.finalizing_start_timestamp.checked_add(feed.update_interval.into()).unwrap()) < timestamp;
                require!(feed_not_finalizing, StarboardError::FinalizingInProgress);
            }
        };

        feed.finalizing_round = Some(round_key);
        round.current_stage = 4;
        round.finalizing_start_timestamp = timestamp;
        Ok(())
    }

    pub fn finalize(ctx: Context<Finalize>) -> Result<()> {
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
    pub feed: Pubkey,
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
    pub token_account: Pubkey,
    pub staker: Pubkey,
    pub feed: Pubkey,
    pub bump: u8,
    pub timestamp: u64,
}

#[account(zero_copy)]
pub struct ReportRecord {
    pub timestamp: u64,
    pub round_height: u64,
    pub round: Pubkey,
    pub feed: Pubkey,
    pub reporter: Pubkey,
    pub escrow: Pubkey,
    // note: fix the types here
    /* confidence interval could be a percentage
     * or we could do it the Pyth way.
     */
    pub value: u64,
    pub confidence_interval: u8,
}

#[account(zero_copy)]
pub struct CertRecord {
    pub timestamp: u64,
    pub round_height: u64,
    pub round: Pubkey,
    pub feed: Pubkey,
    pub reporter: Pubkey,
    pub escrow: Pubkey,
    // note: fix the types here
    /* confidence interval could be a percentage
     * or we could do it the Pyth way.
     */
    pub value: u64,
    pub confidence_interval: u8,
}

#[error_code]
pub enum StarboardError {
    #[msg("No rounds initialized for the feed")]
    NoRoundsError,
    #[msg("Round not ready to change state")]
    RoundNotReady,
    #[msg("Staking for this feed already in progress")]
    StakingInProgress,
    #[msg("Reporting for this feed already in progress")]
    ReportingInProgress,
    #[msg("Comitting for this feed already in progress")]
    ComittingInProgress,
    #[msg("Certifying for this feed already in progress")]
    CertifyingInProgress,
    #[msg("Finalizing for this feed already in progress")]
    FinalizingInProgress,
    #[msg("Cannot stake on this round")]
    RoundNotStaking,
}
