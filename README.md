# MarketView 3.3 — Adrian Vale

A market research and risk-planning desktop workspace, rebuilt with React, TypeScript and Tauri 2. The previous Python UI and application implementation are not included.

## Core features
- 50-symbol starter US stock and ETF watchlist, with editable folders and research notes
- Alpaca market-data integration with real-time IEX intraday bars and daily technical history
- Scanner with daily and 20-day change, EMA 20/50, RSI 14, MACD line, ATR 14 and volume versus its 20-day average
- Transparent illustrative technical checklist score, caution flags, filters and sorting
- Saved scan snapshots (latest 20)
- Price-level alerts checked when quotes are refreshed
- Position-size, risk-amount and reward-to-risk calculator
- Trade journal
- Sydney, Tokyo, London and New York forex-session references plus London, US, Tokyo and Hong Kong exchange hours, with UK GMT/BST conversion
- Full JSON backup export, CSV export and a confirmed local-research-data deletion control
- Classy blue and Royal gold appearance themes

## Run locally
```bash
npm install
npm run tauri dev
```

## Test and build
```bash
npm test
npm run build
npm run tauri build
```

## Data, market data and risk
Watchlists, scan snapshots, alerts, plans and journal entries are stored locally in the app. Market prices and intraday bars are requested from Alpaca by the native desktop backend. The app uses Alpaca's real-time IEX feed by default; IEX covers trades reported on that exchange only. Full US exchange coverage (SIP) requires an eligible paid Alpaca market-data subscription. API credentials are stored in the operating system credential manager, not in app preferences. Data may still be unavailable outside market hours, for unsupported symbols, or during provider interruptions. MarketView does not connect to a brokerage account and the app does not place orders. Technical scores, indicators and position sizing are research aids, not predictions or financial advice. Confirm prices and decisions with your broker or exchange.

## Technology
React 18 · TypeScript · Vite · Tauri 2 · Rust · reqwest · Alpaca Market Data API

Website: https://adrianvaleitl-debug.github.io/Adrian-Vales-Projects/
Discord: https://discord.gg/3TQcf6DwvT

## MarketView 3.3.0

- Added a first-launch investment-risk and no-financial-advice acknowledgement.
- Added configurable links to open watchlist and scanner symbols in a preferred trading platform.
- Renamed the product MarketView and introduced a matching royal-gold M app icon.
