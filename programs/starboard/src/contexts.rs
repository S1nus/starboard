use crate::*;

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
        space = 80,
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
#[instruction(num: u8)]
pub struct StartStaking<'info> {
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        mut,
        seeds = [
            ROUND_SEED,
            feed.key().as_ref(),
            &[num]
        ],
        bump
    )]
    pub round: AccountLoader<'info, Round>,
}

#[derive(Accounts)]
pub struct StartReporting {}

#[derive(Accounts)]
pub struct StartComitting {}

#[derive(Accounts)]
pub struct StartCertifying {}

#[derive(Accounts)]
pub struct StartFinalizing {
}

#[derive(Accounts)]
#[instruction(num: u8, round_height: u64)]
pub struct Stake<'info> {
    #[account(
        init,
        seeds = [
            ESCROW_SEED,
            voter.key().as_ref(),
            feed.key().as_ref(),
            &round_height.to_le_bytes()
        ],
        bump,
        space = 8 + (32*2) + 1 + 8 + 8,
        payer = voter,
    )]
    pub escrow: AccountLoader<'info, Escrow>,
    #[account(mut)]
    pub voter: Signer<'info>,
    #[account(mut)]
    pub feed: AccountLoader<'info, Feed>,
    #[account(
        mut,
        seeds = [
            ROUND_SEED,
            feed.key().as_ref(),
            &[num],
        ],
        bump,
        constraint = 
            round.load()?.round_height == round_height
    )]
    pub round: AccountLoader<'info, Round>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Report {}

#[derive(Accounts)]
pub struct Comitt {}

#[derive(Accounts)]
pub struct Certify {}

#[derive(Accounts)]
pub struct Finalize {}
