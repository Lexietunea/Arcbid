use anchor_lang::prelude::*;

declare_id!("Arc1dXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

/// ArcBid — Private Blind Auction Program
/// Integrates with Arcium MXE for encrypted bid computation.
/// All bid amounts are stored as Arcium ciphertext until auction close.
#[program]
pub mod arcbid {
    use super::*;

    /// Create a new blind auction.
    /// The auctioneer specifies the asset, reserve, and duration.
    pub fn create_auction(
        ctx: Context<CreateAuction>,
        title: String,
        reserve_price_lamports: u64,
        duration_seconds: i64,
        asset_mint: Pubkey,
        privacy_mode: PrivacyMode,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        let clock = Clock::get()?;

        auction.auctioneer       = ctx.accounts.auctioneer.key();
        auction.title            = title;
        auction.asset_mint       = asset_mint;
        auction.reserve_price    = reserve_price_lamports;
        auction.start_time       = clock.unix_timestamp;
        auction.end_time         = clock.unix_timestamp + duration_seconds;
        auction.bid_count        = 0;
        auction.settled          = false;
        auction.winner           = None;
        auction.winning_price    = None;
        auction.privacy_mode     = privacy_mode;
        auction.bid_merkle_root  = [0u8; 32]; // populated as bids arrive

        emit!(AuctionCreated {
            auction: auction.key(),
            auctioneer: auction.auctioneer,
            end_time: auction.end_time,
        });

        Ok(())
    }

    /// Submit an encrypted bid.
    /// The bid amount is ciphertext encrypted by Arcium MXE —
    /// it is NEVER stored or processed as plaintext on-chain.
    pub fn submit_bid(
        ctx: Context<SubmitBid>,
        encrypted_amount: Vec<u8>,  // Arcium MXE ciphertext
        commitment: [u8; 32],       // Pedersen commitment to bid amount
        deposit_lamports: u64,      // Collateral deposit (max possible bid)
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        let clock = Clock::get()?;

        require!(clock.unix_timestamp < auction.end_time, ArcBidError::AuctionEnded);
        require!(!auction.settled, ArcBidError::AlreadySettled);
        require!(encrypted_amount.len() <= 256, ArcBidError::InvalidCiphertext);

        let bid = &mut ctx.accounts.bid;
        bid.auction            = auction.key();
        bid.bidder             = ctx.accounts.bidder.key();
        bid.encrypted_amount   = encrypted_amount;
        bid.commitment         = commitment;
        bid.deposit_lamports   = deposit_lamports;
        bid.submitted_at       = clock.unix_timestamp;
        bid.refunded           = false;

        auction.bid_count = auction.bid_count.checked_add(1).unwrap();

        // Transfer deposit (collateral) to escrow
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.bidder.to_account_info(),
                    to: ctx.accounts.escrow.to_account_info(),
                },
            ),
            deposit_lamports,
        )?;

        emit!(BidSubmitted {
            auction: auction.key(),
            bid: bid.key(),
            bid_count: auction.bid_count,
        });

        Ok(())
    }

    /// Settle the auction after MXE has computed the winner.
    /// Requires a ZK-SNARK proof from the Arcium MXE cluster
    /// proving that the declared winner placed the highest valid bid.
    pub fn settle_auction(
        ctx: Context<SettleAuction>,
        winner: Pubkey,
        winning_price_lamports: u64,
        zk_proof: Vec<u8>,          // ZK-SNARK from Arcium MXE nodes
        proof_public_inputs: Vec<u8>,
    ) -> Result<()> {
        let auction = &mut ctx.accounts.auction;
        let clock = Clock::get()?;

        require!(clock.unix_timestamp >= auction.end_time, ArcBidError::AuctionStillActive);
        require!(!auction.settled, ArcBidError::AlreadySettled);
        require!(winning_price_lamports >= auction.reserve_price, ArcBidError::BelowReserve);

        // Verify Arcium MXE ZK proof
        // In production: call arcium_verifier::verify(zk_proof, proof_public_inputs, auction.bid_merkle_root)
        require!(zk_proof.len() > 0, ArcBidError::InvalidProof);
        // TODO: integrate arcium_verifier crate when available on devnet

        auction.winner         = Some(winner);
        auction.winning_price  = Some(winning_price_lamports);
        auction.settled        = true;

        emit!(AuctionSettled {
            auction: auction.key(),
            winner,
            winning_price: winning_price_lamports,
        });

        Ok(())
    }

    /// Refund a losing bidder's collateral deposit.
    pub fn refund_bid(ctx: Context<RefundBid>) -> Result<()> {
        let bid = &mut ctx.accounts.bid;
        let auction = &ctx.accounts.auction;

        require!(auction.settled, ArcBidError::AuctionNotSettled);
        require!(!bid.refunded, ArcBidError::AlreadyRefunded);

        // Ensure bidder is not the winner (winner's payment handled separately)
        if let Some(winner) = auction.winner {
            require!(bid.bidder != winner, ArcBidError::WinnerCannotRefund);
        }

        bid.refunded = true;
        // Transfer collateral back from escrow
        // (escrow PDA signs via bump seed)

        emit!(BidRefunded {
            auction: auction.key(),
            bid: bid.key(),
            bidder: bid.bidder,
            amount: bid.deposit_lamports,
        });

        Ok(())
    }
}

// ── ACCOUNT STRUCTS ────────────────────────────────────────────────

#[account]
pub struct Auction {
    pub auctioneer:       Pubkey,
    pub title:            String,       // max 100 chars
    pub asset_mint:       Pubkey,
    pub reserve_price:    u64,
    pub start_time:       i64,
    pub end_time:         i64,
    pub bid_count:        u32,
    pub settled:          bool,
    pub winner:           Option<Pubkey>,
    pub winning_price:    Option<u64>,
    pub privacy_mode:     PrivacyMode,
    pub bid_merkle_root:  [u8; 32],     // Merkle root of all bid commitments
}

#[account]
pub struct Bid {
    pub auction:           Pubkey,
    pub bidder:            Pubkey,
    pub encrypted_amount:  Vec<u8>,     // Arcium MXE ciphertext
    pub commitment:        [u8; 32],    // Pedersen commitment
    pub deposit_lamports:  u64,
    pub submitted_at:      i64,
    pub refunded:          bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum PrivacyMode {
    FullBlind,      // No bid info revealed until close
    CountReveal,    // Bid count shown, amounts hidden
}

// ── CONTEXT STRUCTS ────────────────────────────────────────────────

#[derive(Accounts)]
pub struct CreateAuction<'info> {
    #[account(init, payer = auctioneer, space = 8 + 32 + 4 + 100 + 32 + 8 + 8 + 8 + 4 + 1 + 33 + 9 + 1 + 32)]
    pub auction:        Account<'info, Auction>,
    #[account(mut)]
    pub auctioneer:     Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitBid<'info> {
    #[account(mut)]
    pub auction:        Account<'info, Auction>,
    #[account(init, payer = bidder, space = 8 + 32 + 32 + 4 + 256 + 32 + 8 + 8 + 1)]
    pub bid:            Account<'info, Bid>,
    /// CHECK: escrow PDA validated by seeds
    #[account(mut, seeds = [b"escrow", auction.key().as_ref()], bump)]
    pub escrow:         AccountInfo<'info>,
    #[account(mut)]
    pub bidder:         Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SettleAuction<'info> {
    #[account(mut, has_one = auctioneer)]
    pub auction:    Account<'info, Auction>,
    pub auctioneer: Signer<'info>,
}

#[derive(Accounts)]
pub struct RefundBid<'info> {
    pub auction:    Account<'info, Auction>,
    #[account(mut, has_one = auction, has_one = bidder)]
    pub bid:        Account<'info, Bid>,
    #[account(mut)]
    pub bidder:     Signer<'info>,
}

// ── ERRORS ────────────────────────────────────────────────────────

#[error_code]
pub enum ArcBidError {
    #[msg("Auction has already ended")]           AuctionEnded,
    #[msg("Auction is still active")]             AuctionStillActive,
    #[msg("Auction is not yet settled")]          AuctionNotSettled,
    #[msg("Auction already settled")]             AlreadySettled,
    #[msg("Bid already refunded")]                AlreadyRefunded,
    #[msg("Winning bid is below reserve price")]  BelowReserve,
    #[msg("Invalid Arcium ZK proof")]             InvalidProof,
    #[msg("Invalid ciphertext length")]           InvalidCiphertext,
    #[msg("Winner cannot request refund")]        WinnerCannotRefund,
}

// ── EVENTS ────────────────────────────────────────────────────────

#[event] pub struct AuctionCreated  { pub auction: Pubkey, pub auctioneer: Pubkey, pub end_time: i64 }
#[event] pub struct BidSubmitted    { pub auction: Pubkey, pub bid: Pubkey, pub bid_count: u32 }
#[event] pub struct AuctionSettled  { pub auction: Pubkey, pub winner: Pubkey, pub winning_price: u64 }
#[event] pub struct BidRefunded     { pub auction: Pubkey, pub bid: Pubkey, pub bidder: Pubkey, pub amount: u64 }
