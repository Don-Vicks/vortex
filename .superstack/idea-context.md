# Idea Context

## Idea
TradeGuard Pro: SaaS that enforces trading rules on MT5 accounts through MetaApi. Directions considered: prop firm in a box, plus verified track records and USDC payouts on Solana.

```json
{
  "validation": {
    "demand_signals": [
      "White-label prop platforms charge $5k-50k to set up plus $2k-15k/mo (Track360, 2026)",
      "Paid B2C discipline tools are live: LockMyTrades $29.99/mo, TradeLock",
      "Long-running MQL5/ForexFactory threads asking how to lock an account after a loss",
      "Prop-firm crypto payouts reached $115M in Q1 2026, up 129% year on year (PropFirmMap)",
      "Gap: no direct user evidence for TradeGuard yet"
    ],
    "risks": [
      {"category": "market", "description": "MetaApi sells its own Risk Management API to prop firms", "severity": "high"},
      {"category": "platform", "description": "MetaQuotes license revocations and the move to DXtrade shrink the MT5-only market", "severity": "high"},
      {"category": "technical", "description": "MetaApi enforcement happens after the trade (can close, cannot block); competitors advertise pre-trade blocking", "severity": "medium"},
      {"category": "trust", "description": "Users must share their MT5 master password to enable enforcement", "severity": "medium"},
      {"category": "unit-economics", "description": "MetaApi charges per account, so costs grow linearly with users", "severity": "medium"},
      {"category": "regulatory", "description": "Features that look like fund management or staking bring regulatory exposure", "severity": "medium"}
    ],
    "go_no_go": "pivot",
    "confidence": 0.5,
    "next_steps": [
      "Wedge: rule enforcement plus mentor dashboard for trading educators and signal communities, priced per seat",
      "Contact 20 educators or community owners and hold 8-10 problem interviews in week 1",
      "Put up a landing page with real pricing; target 2+ paid pilots or LOIs",
      "Interview 10 funded traders comparing TradeGuard with LockMyTrades",
      "Integrate MetaApi Risk Management API instead of custom drawdown trackers",
      "Defer Solana: use Solana Attestation Service for track records and an existing USDC rail for payouts, only once users ask",
      "Kill bar: 0 pilots after 20 conversations, then fall back to a multi-platform rule-engine API"
    ]
  },
  "landscape": {
    "direct_competitors": [
      {"name": "MetaApi Risk Management API", "url": "https://metaapi.cloud/docs/risk-management/", "status": "live", "strength": "Challenge and drawdown trackers sold directly to prop firms", "weakness": "API only, no MT5 netting support, no trader-facing UX"},
      {"name": "PropFirmAI / Swiset / FXTrusts", "url": "https://prop-firm.ai/", "status": "live", "strength": "Turnkey white-label prop platform on multiple trading platforms", "weakness": "Commoditized, generic risk engines"},
      {"name": "LockMyTrades", "url": "https://www.lockmytrades.com/", "status": "live", "strength": "Pre-trade blocking, revenge guard, news embargo, $29.99/mo", "weakness": "Solo traders only, no community layer"},
      {"name": "TradeLock", "url": "https://tradeily.com/", "status": "live", "strength": "Platform lock with a 1-hour emergency unlock", "weakness": "Solo traders only"},
      {"name": "Hyro Protocol", "url": "https://www.hyrotrader.com/blog/hyro-protocol-on-chain-crypto-prop-trading/", "status": "live (Jul 2026, Solana)", "strength": "Established prop brand, verifiable track records, USDC", "weakness": "Crypto trading only, not MT5"}
    ],
    "substitutes": [
      {"name": "Hand-written MQL5 EAs", "approach": "Enforcement inside the terminal", "why_users_stay": "Free, and familiar to MT5 users"},
      {"name": "Myfxbook / trading journals", "approach": "Analytics after the fact", "why_users_stay": "Established, and used as social proof"},
      {"name": "Rise", "approach": "Stablecoin payout rails for prop firms", "why_users_stay": "$1.5B+ volume, already used by most futures prop firms"}
    ],
    "dead_projects": [
      {"name": "True Forex Funds and 80-100 other prop firms", "why_failed": "MetaQuotes revoked their MT4/MT5 licenses (Feb 2024 onward), plus weak business models"}
    ],
    "crowdedness": "crowded",
    "moat_type": "distribution / switching costs (educator cohorts), then a data moat",
    "differentiation": "Rule compliance for groups of traders (educators and signal communities), not for individual traders or prop firms"
  }
}
```
