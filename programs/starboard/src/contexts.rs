use crate::*;
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

#[derive(Accounts)]
#[instruction(id: [u8; 32], description: [u8; 32], update_interval: u32)]
pub struct InitFeed<'info> {
    #[account(
        init,
        seeds=[
            FEED_SEED,
            id.as_ref(),
        ],
        bump,
        payer = payer,
        space = (32*2) +(33*5)+ (8*2) + 4 + 8 + 1 + 8 + 8 + 8
    )]
    pub feed: AccountLoader<'info, Feed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(num: u8)]
pub struct InitRound<'info> {
    #[account(
        init,
        seeds=[
            ROUND_SEED,
            feed.key().as_ref(),
            &[num]
        ],
        bump,
        payer = payer,
        space = 80+32,
    )]
    pub round: AccountLoader<'info, Round>,
    #[account(
        mut,
        constraint = !feed.load()?.started,
    )]
    pub feed: AccountLoader<'info, Feed>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StartFeed<'info> {
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
}

#[derive(Accounts)]
pub struct StartStaking<'info> {
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        mut,
        has_one = feed
    )]
    pub round: AccountLoader<'info, Round>,
}

#[derive(Accounts)]
pub struct StartReporting<'info> {
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        mut,
        has_one = feed
    )]
    pub round: AccountLoader<'info, Round>,
}

#[derive(Accounts)]
pub struct StartComitting<'info> {
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        mut,
        has_one = feed
    )]
    pub round: AccountLoader<'info, Round>,
}

#[derive(Accounts)]
pub struct StartCertifying<'info> {
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        mut,
        has_one = feed
    )]
    pub round: AccountLoader<'info, Round>,
}

#[derive(Accounts)]
pub struct StartFinalizing {
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(
        init,
        seeds = [
            ESCROW_SEED,
            voter.key().as_ref(),
            round.key().as_ref(),
        ],
        bump,
        space = 96+32,
        payer = voter,
    )]
    pub escrow: AccountLoader<'info, Escrow>,
    #[account(
        init,
        seeds = [
            ESCROW_TOKEN_SEED,
            escrow.key().as_ref()
        ],
        bump,
        payer = voter,
        token::mint = native_mint,
        token::authority = program_as_signer,
    )]
    pub escrow_token: Account<'info, TokenAccount>,
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(
        mut,
        token::mint = native_mint,
        token::authority = voter,
    )]
    pub voter_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        mut,
        has_one = feed
    )]
    pub round: AccountLoader<'info, Round>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    #[account(address=NATIVE_MINT)]
    pub native_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    /// CHECK: program as signer
    #[account(seeds=[b"program",b"signer"], bump)]
    pub program_as_signer: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Report<'info> {
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        has_one=feed,
    )]
    pub round: AccountLoader<'info, Round>,
    #[account(mut)]
    pub reporter: Signer<'info>,
    #[account(
        mut,
        token::mint = native_mint,
        token::authority = reporter,
    )]
    pub reporter_token: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds=[
            REPORT_RECORD_SEED,
            round.key().as_ref(),
            reporter.key().as_ref(),
        ],
        bump,
        payer=reporter,
        space=8+8+32+32+32+32+8+1+8,
    )]
    pub report_record: AccountLoader<'info, ReportRecord>,
    #[account(
        init,
        seeds=[
            ESCROW_TOKEN_SEED,
            report_record.key().as_ref(),
        ],
        bump,
        payer = reporter,
        token::mint = native_mint,
        token::authority = program_as_signer,
    )]
    pub report_escrow: Account<'info, TokenAccount>,
    #[account(address=NATIVE_MINT)]
    pub native_mint: Account<'info, Mint>,
    /// CHECK: program as signer
    #[account(seeds=[b"program",b"signer"], bump)]
    pub program_as_signer: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Comitt<'info> {
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        has_one=feed,
    )]
    pub round: AccountLoader<'info, Round>,
    #[account(
        mut,
        has_one=staker,
    )]
    pub escrow: AccountLoader<'info, Escrow>,
    pub staker: Signer<'info>,
}

#[derive(Accounts)]
pub struct Certify<'info> {
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        has_one=feed,
    )]
    pub round: AccountLoader<'info, Round>,
    #[account(
        has_one=round,
    )]
    pub report_record: AccountLoader<'info, ReportRecord>,
    #[account(mut)]
    pub certifier: Signer<'info>,
    #[account(
        mut,
        token::mint = native_mint,
        token::authority = certifier
    )]
    pub certifier_tokens: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds=[
            ESCROW_TOKEN_SEED,
            cert_record.key().as_ref(),
        ],
        bump,
        payer = certifier,
        token::mint = native_mint,
        token::authority = program_as_signer,
    )]
    pub certifier_escrow: Account<'info, TokenAccount>,
    #[account(
        init,
        seeds=[
            CERT_RECORD_SEED,
            report_record.key().as_ref(),
            certifier.key().as_ref(),
        ],
        bump,
        payer=certifier,
        space=8+8+32+32+32+32+8+1+8,
    )]
    pub cert_record: AccountLoader<'info, CertRecord>,
    /// CHECK: program as signer
    #[account(seeds=[b"program",b"signer"], bump)]
    pub program_as_signer: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
    pub native_mint: Account<'info, Mint>,
}

#[derive(Accounts)]
pub struct Finalize {}
