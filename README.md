<div align="center">
  <a href="https://github.com/Sensible-Analytics/folio">
    <img src="apps/frontend/public/logo.svg" alt="Logo" width="80" height="80">
  </a>

  <h3 align="center">Folio</h3>

  <p align="center">
    Wealthfolio + Australian Bank Statement Downloader
    <br />
    <em>A community fork of <a href="https://github.com/afadil/wealthfolio">afadil/wealthfolio</a></em>
    <br />
    <br />
    <a href="https://github.com/afadil/wealthfolio/releases">Download Wealthfolio</a>
    ·
    <a href="https://wealthfolio.app">Official Website</a>
    ·
    <a href="https://github.com/Sensible-Analytics/folio/issues">Report Issues</a>
  </p>
</div>

---

## Demo

<!-- Replace the URL below once you record a screen capture and upload it to YouTube/Loom -->

[![Folio – Bank Connect walkthrough](https://img.shields.io/badge/▶_Watch_Tutorial-YouTube-red?style=for-the-badge&logo=youtube)](https://github.com/Sensible-Analytics/folio#tutorial)

> **Tutorial** — complete written walkthrough is in the [Tutorial](#tutorial)
> section below.

---

## What is this?

This is a community fork of [Wealthfolio](https://github.com/afadil/wealthfolio)
— the local-first, open-source desktop investment tracker — with one extra
feature added:

**Bank Connect**: automatically download bank statements (PDF) from Australian
banks directly inside the Wealthfolio desktop app.

Everything else — portfolio tracking, performance analytics, activities, goals —
is the same as the upstream project.

---

## Bank Connect — Australian Bank Statement Downloader

The Bank Connect feature lets you log in to your Australian bank's website
inside Wealthfolio and download statements without leaving the app.

### Supported Banks

| Bank              | Login URL                       |
| ----------------- | ------------------------------- |
| ING Australia     | ing.com.au                      |
| CommBank (CBA)    | netbank.com.au                  |
| ANZ               | anz.com.au                      |
| Bank of Melbourne | ibanking.bankofmelbourne.com.au |
| Beyond Bank       | ibank.beyondbank.com.au         |

### How it works

1. Go to **Bank Connect** in the sidebar
2. Click **Open Login** for your bank — a browser window opens at the bank's
   real login page
3. Log in normally (two-factor, biometrics, whatever your bank requires)
4. Once logged in, click **Start Download** — the app collects your statement
   download links
5. PDFs are saved to `~/BankStatements/{bank}/` on your computer

Your login session is stored locally (per bank, isolated). No credentials are
sent anywhere — the app only talks directly to your bank's website.

### Settings

In **Settings → Bank Connect** you can configure:

- Download folder (default: `~/BankStatements`)
- How many years back to download (default: 7, max 10)
- Which banks are enabled
- Whether to overwrite existing files

---

## Relationship to the upstream project

This fork tracks the `main` branch of
[afadil/wealthfolio](https://github.com/afadil/wealthfolio). The only changes on
top of upstream are:

| Change               | Description                                                                                      |
| -------------------- | ------------------------------------------------------------------------------------------------ |
| `feat(bank-connect)` | The Bank Connect feature described above                                                         |
| `ci: test suite`     | Multi-layer test suite (property tests, Kani formal proofs, migration integrity, Playwright E2E) |
| `docs(claude)`       | Development notes for AI-assisted development                                                    |

If you want the base investment tracker without Bank Connect, use the official
app: [wealthfolio.app](https://wealthfolio.app/).

---

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) ≥ 20
- [pnpm](https://pnpm.io/) ≥ 9
- [Rust](https://www.rust-lang.org/) (stable toolchain)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS

### Build from source

```bash
# Clone this fork
git clone https://github.com/Sensible-Analytics/folio.git
cd folio

# Install Node dependencies
pnpm install

# Run in development mode (opens the desktop app)
pnpm tauri dev
```

### Run tests

```bash
# TypeScript tests
pnpm test

# Rust tests
cargo test

# Type check
pnpm type-check
```

---

## Wealthfolio — Original Features

Everything below is from the upstream
[Wealthfolio](https://github.com/afadil/wealthfolio) project.

### Key Features

- **Portfolio Tracking** — Track investments across multiple accounts and asset
  types
- **Performance Analytics** — Detailed performance metrics and historical
  analysis
- **Activity Management** — Import and manage all trading activities
- **Goal Planning** — Set and track financial goals with allocation management
- **Local Data** — All data stored locally with no cloud dependencies
- **Multi-Currency** — Support for multiple currencies with exchange rate
  management
- **Cross-Platform** — Available on Windows, macOS, and Linux
- **Extensible** — Powerful addon system for custom functionality

### Web Mode

Run Wealthfolio in a browser via a local Axum server:

```bash
pnpm run dev:web
```

See the
[original README](https://github.com/afadil/wealthfolio/blob/main/README.md) for
full web mode and Docker documentation.

### Folder Structure

```
folio/
├── apps/frontend/          # React + Vite frontend
├── apps/tauri/             # Tauri desktop app (Rust IPC commands)
│   └── src/banks/          # Bank automation scripts (Bank Connect)
├── apps/server/            # Axum HTTP server (web mode)
├── crates/core/            # Business logic, models, services
│   └── src/bank_connect/   # Bank Connect core logic
├── crates/storage-sqlite/  # SQLite storage (Diesel ORM, migrations)
├── addons/                 # Example addons
├── packages/               # Shared TypeScript packages
└── docs/                   # Documentation
    └── testing/            # Testing strategy and learnings
```

---

## Tutorial

This tutorial walks you through installing Folio, tracking a portfolio, and
downloading Australian bank statements from scratch.

### Step 1 — Install

```bash
# Prerequisites: Node ≥ 20, pnpm ≥ 9, Rust stable
# https://tauri.app/start/prerequisites/

git clone https://github.com/Sensible-Analytics/folio.git
cd folio
pnpm install
pnpm tauri dev          # opens the desktop app
```

The first `pnpm tauri dev` compiles Rust (~2–5 min). Subsequent runs are fast.

---

### Step 2 — Add your first account

```
App → Accounts → + New Account
  Name:     My Broker
  Type:     Securities
  Currency: AUD
```

Folio stores everything locally in SQLite — nothing leaves your machine.

---

### Step 3 — Import activities (trades, dividends, …)

```
Activities → Import
  ↳ Drop a CSV or use "Add Activity" for manual entry
```

Supported activity types: `BUY`, `SELL`, `DIVIDEND`, `INTEREST`, `DEPOSIT`,
`WITHDRAWAL`, `FEE`, `TAX`, `SPLIT`, and more.

After import the **Dashboard** updates instantly with portfolio value,
performance chart, and allocation breakdown.

---

### Step 4 — Download Australian bank statements (Bank Connect)

```
Sidebar → Bank Connect
```

```
┌────────────────────────────────────────────────────────────┐
│  Bank Connect                                              │
├──────────────────────┬─────────────────────────────────────┤
│  [ING]  [CBA]        │  Log                                │
│  [ANZ]  [BOM]        │  10:42  INFO  Opening ING window…   │
│  [Beyond]            │  10:43  INFO  Login detected ✓      │
│                      │  10:43  INFO  Downloading…          │
│                      │  10:44  OK    12 files saved        │
└──────────────────────┴─────────────────────────────────────┘
```

1. Click **Open Login** next to your bank.
2. A browser window opens at the bank's real login URL — log in normally.
3. Once logged in, click **Start Download**.
4. PDFs land in `~/BankStatements/{bank}/`.

> Your credentials never touch Folio — the embedded browser talks directly to
> the bank.

---

### Step 5 — Configure Bank Connect settings

```
Settings → Bank Connect
  Download folder:  ~/BankStatements   (change to any path)
  Years back:       7                  (1 – 10)
  Enabled banks:    ☑ ING  ☑ CBA  …
  Overwrite files:  off
```

---

### Step 6 — Track goals

```
Goals → + New Goal
  Name:       Emergency Fund
  Target:     $20,000 AUD
  Allocate accounts → link Cash accounts
```

The goal card shows current value vs target and a progress bar that updates
whenever new activities are imported.

---

### Keyboard shortcuts

| Action               | Shortcut       |
| -------------------- | -------------- |
| Open command palette | `Cmd/Ctrl + K` |
| New activity         | `Cmd/Ctrl + N` |
| Refresh quotes       | `Cmd/Ctrl + R` |
| Toggle sidebar       | `Cmd/Ctrl + B` |

---

### Recording your own demo

If you want to record a screen capture and embed it here, a few good tools:

- **macOS**: `Cmd + Shift + 5` → record screen → upload to YouTube/Loom
- **[VHS](https://github.com/charmbracelet/vhs)**: terminal-only GIF recorder
- **[Loom](https://www.loom.com)**: quick shareable link, thumbnail embeds in
  GitHub

Once you have a YouTube or Loom URL, replace the `#tutorial` link in the
[Demo](#demo) badge at the top of this file.

---

## Contributing

This fork is maintained by
[Sensible Analytics](https://github.com/Sensible-Analytics).

For issues with the Bank Connect feature, open an issue here.

For issues with the core investment tracker, consider contributing upstream to
[afadil/wealthfolio](https://github.com/afadil/wealthfolio).

---

## License

Code is licensed under **AGPL-3.0** — same as the upstream project. See
`LICENSE`.

Wealthfolio and the Wealthfolio logo are trademarks of Teymz Inc. See
[TRADEMARKS.md](TRADEMARKS.md).
