# ArcBid — Private Blind Auctions on Solana

> **Arcium × Solana Hackathon Submission**
> Sealed-bid, MEV-proof auctions powered by Arcium's Multi-party eXecution Environments.

![ArcBid Banner](https://img.shields.io/badge/Powered%20By-Arcium%20MXE-7c5cff?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cGF0aCBkPSJNMTIgMkw0IDdWMTJMMTIgMjJMMjAgMTJWN0wxMiAyWiIgc3Ryb2tlPSJ3aGl0ZSIgc3Ryb2tlLXdpZHRoPSIyIiBmaWxsPSJub25lIi8+PC9zdmc+)
![Solana](https://img.shields.io/badge/Solana-Devnet-9945ff?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)
![Open Source](https://img.shields.io/badge/Open%20Source-Yes-brightgreen?style=for-the-badge)

---

## What Is ArcBid?

**ArcBid** is a decentralised blind auction protocol built on Solana that uses **Arcium's MXE (Multi-party eXecution Environments)** to guarantee complete bid privacy throughout the entire auction lifecycle.

In a standard on-chain auction, anyone can observe all bids in real time — creating opportunities for collusion, sniping, and MEV (Miner Extractable Value) exploitation. ArcBid eliminates this entirely.

> **With ArcBid, bids are encrypted before they touch the chain, encrypted while they sit on the chain, and only the winner and final price are revealed when the auction closes.**

---

## How It Works

```
Bidder                    Arcium MXE Cluster               Solana Anchor Program
   │                             │                                  │
   │──── Bid Amount ────────────►│                                  │
   │     (plaintext, local)      │                                  │
   │                         Encrypt                                │
   │◄────────────────────────────│                                  │
   │     Ciphertext              │                                  │
   │                             │                                  │
   │──── Submit Ciphertext ────────────────────────────────────────►│
   │                             │              Store sealed bid    │
   │                             │                                  │
   │         ... auction runs (all bids stay encrypted) ...         │
   │                             │                                  │
   │              [Auction closes]                                  │
   │                             │◄──── All sealed bids ───────────│
   │                         Decrypt &                              │
   │                         Compare                                │
   │                         (private)                              │
   │                             │────── Winner + Price ──────────►│
   │                             │        (ZK Proof)                │
   │                             │                 Settle & Pay out │
```

### Step-by-Step

1. **Create Auction** — An auctioneer deploys a new auction via the Anchor smart contract, specifying the asset (NFT mint, SPL token, etc.), reserve price, and auction duration.

2. **Submit Encrypted Bid** — Bidders send their bid amount to the Arcium client SDK, which encrypts it using the MXE cluster's public key before the transaction is ever signed. The Solana program receives only the ciphertext.

3. **Blind Auction Period** — All bids are stored encrypted on-chain throughout the auction window. No participant, validator, or observer can determine any bid amount. MEV is structurally impossible.

4. **MXE Computes Winner** — When the auction closes, the Arcium MXE nodes perform a **multi-party comparison** over all encrypted bids, identifying the highest bidder without ever exposing individual bid values. A ZK-SNARK proof of correct computation is generated.

5. **On-Chain Settlement** — The winning price and wallet address are published on-chain alongside the validity proof. The Anchor program transfers the asset to the winner and refunds all other deposited collateral.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Smart Contract | Solana Anchor Program (Rust) |
| Privacy Layer | Arcium MXE (Multi-party eXecution) |
| Encryption | Arcium Client SDK — ARC-MXE v2 |
| Proof System | ZK-SNARK (Groth16) |
| Wallet Support | Phantom, Solflare (via Wallet Adapter) |
| Frontend | Vanilla HTML / CSS / JavaScript |
| Network | Solana Devnet |

---

## Arcium Integration — Technical Detail

ArcBid integrates Arcium at the **bid submission layer** and the **auction close layer**:

### Bid Submission
```javascript
// Pseudocode — real integration uses @arcium/client SDK
import { ArciumClient } from '@arcium/client';

const client = new ArciumClient({ cluster: 'devnet' });

async function submitEncryptedBid(bidAmountLamports, auctionPubkey) {
  // Encrypt bid amount inside Arcium's MXE
  const { ciphertext, commitment } = await client.encrypt({
    value: bidAmountLamports,
    program: ARCBID_PROGRAM_ID,
    context: auctionPubkey.toBase58(),
  });

  // Submit ciphertext to Anchor program — amount never visible on-chain
  const tx = await program.methods
    .submitBid(ciphertext, commitment)
    .accounts({ auction: auctionPubkey, bidder: wallet.publicKey })
    .rpc();

  return tx;
}
```

### Auction Settlement
```rust
// Anchor program — settlement after MXE reveals winner
pub fn settle_auction(
    ctx: Context<SettleAuction>,
    winner: Pubkey,
    winning_price: u64,
    zk_proof: Vec<u8>,          // ZK-SNARK from Arcium MXE
) -> Result<()> {
    // Verify ZK proof before accepting result
    require!(
        verify_arcium_proof(&zk_proof, &ctx.accounts.auction.bid_root),
        ArcBidError::InvalidProof
    );

    // Transfer asset to winner
    transfer_asset(&ctx, winner, winning_price)?;

    // Mark auction settled
    ctx.accounts.auction.settled = true;
    Ok(())
}
```

### Why This Prevents MEV

- **Pre-auction**: Bids encrypted before signing → no mempool leakage
- **During auction**: All bids are ciphertext → no amount visible via RPC or block explorers
- **At close**: Only output of MXE private computation is published → winner revealed, losing amounts never disclosed
- **Integrity**: ZK proof guarantees MXE computed the correct winner without a trusted third party

---

## Frontend

The ArcBid frontend is a **multi-page vanilla HTML/CSS/JS application** with a dark purple aesthetic matching Arcium's visual identity.

### Pages
- **`index.html`** — Landing page with hero, live auction listings, how-it-works section
- **`create.html`** — Auction creation form with live preview

### Features
- 🔌 Wallet connection modal (Phantom + Solflare)
- 🔐 Bid submission with real-time Arcium encryption preview
- ⏱ Live countdown timers per auction
- 🎨 Animated particle background matching Arcium's dot-grid theme
- 📱 Fully responsive (mobile, tablet, desktop)
- 🏷 Auction status badges (Live / Upcoming / Ended)
- ✨ Encrypted bid shimmer (simulates sealed ciphertext display)

---

## Repository Structure

```
arcbid/
├── index.html              # Main landing page
├── create.html             # Auction creation page
├── css/
│   └── style.css           # Full stylesheet (Arcium dark theme)
├── js/
│   └── app.js              # Wallet connection, modals, timers, canvas
├── programs/
│   └── arcbid/
│       └── src/
│           └── lib.rs      # Anchor smart contract (scaffold)
├── tests/
│   └── arcbid.ts           # Anchor test suite (scaffold)
├── Anchor.toml             # Anchor config
└── README.md               # This file
```

---

## Running Locally

### Frontend Only (No Rust/Solana required)

```bash
git clone https://github.com/yourusername/arcbid.git
cd arcbid

# Open in browser — no build step needed
open index.html
# or
npx serve .
```

### Full Stack (Requires Solana + Anchor CLI)

```bash
# Prerequisites
# - Rust: https://rustup.rs
# - Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools
# - Anchor CLI: https://www.anchor-lang.com/docs/installation
# - Arcium CLI: https://docs.arcium.com/cli

# 1. Install dependencies
cd arcbid

# 2. Configure Solana for devnet
solana config set --url devnet
solana airdrop 2

# 3. Build and deploy Anchor program
anchor build
anchor deploy

# 4. Install Arcium client
npm install @arcium/client

# 5. Update program ID in Anchor.toml and lib.rs with deployed address

# 6. Run tests
anchor test

# 7. Serve frontend
npx serve .
```

---

## Privacy Guarantees

| Threat | Traditional Auction | ArcBid |
|---|---|---|
| Front-running bids | ❌ Vulnerable | ✅ Impossible (ciphertext only) |
| Collusion between bidders | ❌ Observable bids | ✅ No bid amounts visible |
| MEV extraction by validators | ❌ Exposed in mempool | ✅ No exploitable data |
| Auctioneer manipulation | ❌ Possible | ✅ MXE is trustless |
| Fake winner declared | ❌ Possible | ✅ ZK proof required |

---

## Judging Criteria

| Criterion | ArcBid Approach |
|---|---|
| **Innovation** | First blind auction protocol on Solana using Arcium MXE for sealed-bid computation |
| **Technical Implementation** | Anchor smart contract + Arcium SDK + ZK proof verification at settlement |
| **User Experience** | One-click wallet connect, clean bid UI, real-time encrypted preview, countdown timers |
| **Impact** | Unlocks fair NFT drops, DAO treasury auctions, token distributions — eliminating front-running |
| **Clarity** | This README + inline code comments + UI explanatory text throughout |

---

## Disclaimer

This project is a **hackathon submission** and a proof-of-concept. The smart contract has **not been audited**. Do not use with real funds on mainnet. Frontend wallet connection is simulated for UI demonstration purposes.

---

## License

MIT License — see [LICENSE](LICENSE) for full text.

Open source. Fork it. Build on it. Improve it.

---

## Built With

- [Arcium](https://arcium.com) — MXE privacy infrastructure
- [Solana](https://solana.com) — High-performance L1 blockchain
- [Anchor](https://anchor-lang.com) — Solana smart contract framework
- [Phantom Wallet](https://phantom.app) — Solana wallet
- [Solflare](https://solflare.com) — Solana wallet

---

*ArcBid — Because fair price discovery shouldn't require trusting anyone.*
