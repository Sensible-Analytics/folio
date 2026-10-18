# Sensible Folio — User Manual

Sensible Folio is a local-first personal wealth application built for Australian
investors. It brings your investments (shares, ETFs, managed funds, crypto,
cash) and your bank accounts into one private dashboard, with automatic import
of Australian bank statements, portfolio performance analytics, income tracking,
and tax reporting.

This manual is written from the **user's point of view**: what each feature
does, and how to configure and use it to manage your personal finances.

> **New to Folio?** If you just want to install it and get running, start with
> [Getting Started](./getting-started.md). This manual is the full reference.

---

## Table of contents

1. [Introduction](#1-introduction)
2. [Key concepts](#2-key-concepts)
3. [Feature list at a glance](#3-feature-list-at-a-glance)
4. [Installing and first run](#4-installing-and-first-run)
5. [Configuring Folio](#5-configuring-folio)
6. [Managing your personal finances](#6-managing-your-personal-finances)
7. [Activity types reference](#7-activity-types-reference)
8. [Supported banks and brokers](#8-supported-banks-and-brokers)
9. [Privacy and security](#9-privacy-and-security)
10. [Troubleshooting and FAQ](#10-troubleshooting-and-faq)
11. [Getting help](#11-getting-help)

---

## 1. Introduction

### What Folio is

Folio (distributed as **Sensible Folio**) is a wealth portfolio management
application. Everything — your accounts, transactions, holdings, performance,
and reports — lives in a SQLite database **on your own device**. There is no
cloud account, no subscription, and no third-party server holding your financial
data.

### Who it is for

- Australian investors who want to track shares, ETFs, managed funds, property,
  crypto, and cash in one place.
- People who want to import transactions from Australian banks and brokers
  without handing credentials to a third-party aggregation service.
- Anyone who values privacy: your data stays local unless _you_ explicitly turn
  on encrypted device sync.

### What makes Folio different

- **Local-first.** Your data is stored on your machine. It works offline.
- **Australian-focused.** Bank Connect supports Australian institutions, and
  there is first-class support for importing bank and broker statements.
- **Unified view.** Investments and bank/cash accounts appear in one dashboard,
  so you can see your complete financial picture — not just your brokerage.
- **Extensible.** An addon system lets you add new views and analysis without
  modifying the core app.
- **Private sync.** Optional end-to-end encrypted sync replicates your database
  between _your own_ devices.

---

## 2. Key concepts

Understanding these terms makes the rest of the manual easy to follow.

| Concept                 | What it means in Folio                                                                                                                                              |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Account**             | A single real-world account — a brokerage, bank, or cash account. You can have as many as you have in real life.                                                    |
| **Activity**            | Any event that changes an account: a trade, a deposit, a dividend, a fee, and so on. There are 14 canonical activity types (see [§7](#7-activity-types-reference)). |
| **Holding / Position**  | The quantity of a security you own in an account, derived from your activities.                                                                                     |
| **Portfolio**           | The combined view of all your accounts and holdings.                                                                                                                |
| **Asset & taxonomy**    | A security or instrument you hold, plus the classification system (asset class, sector, region) used for allocation and grouping.                                   |
| **Benchmark**           | An index (for example an ASX index) you compare your portfolio performance against.                                                                                 |
| **Contribution limit**  | A cap on contributions (for example superannuation concessional/non-concessional caps) that you can track against.                                                  |
| **Addon**               | An optional, sandboxed extension that adds new views or features.                                                                                                   |
| **Bank Connect**        | The built-in browser panel that logs into your bank/broker site and imports transactions — desktop app only.                                                        |
| **Wealthfolio Connect** | The optional encrypted sync feature that replicates your data to your own devices.                                                                                  |

---

## 3. Feature list at a glance

### Portfolio and investments

- **Multi-asset tracking** — shares, ETFs, managed funds, property, crypto,
  cash.
- **Holdings view** — current positions per account with cost basis.
- **Holdings insights** — deeper breakdown and analysis of what you hold.
- **Asset profiles** — a per-asset page showing your activity and exposure in
  that instrument.
- **Performance analytics** — time-weighted return (TWR) and money-weighted
  return (IRR).
- **Benchmark comparison** — compare your performance against a chosen index.
- **Allocation charts** — see how your money is split by asset class, sector,
  currency, and taxonomy.

### Income and tax

- **Income tracking** — dividends, interest, and distributions over time.
- **Dividend tracking** — including dividend reinvestment (DRIP) handling.
- **Tax reporting support** — capital gains and income summaries, plus an
  **Exports** screen for taking data out of the app.

### Bank and broker integration

- **Bank Connect** — automatic import from supported Australian banks and
  brokers via an embedded browser panel.
- **Statement import** — import **CSV, OFX, and QIF** files from banks and
  brokers.
- **AI-assisted import** — the AI assistant can help map and import a CSV.
- **Transaction categorisation and cash-flow analysis** — understand where your
  money goes.

### Planning and insight

- **Net worth view** — your complete financial picture across accounts.
- **Goals** — set and track savings/investment targets.
- **Contribution limits** — track contributions against configured caps.
- **Portfolio insights** — analysis screens that surface patterns in your data.
- **Health view** — indicators that summarise the state of your finances.
- **AI assistant** — ask questions about your data in natural language.

### Platform and privacy

- **Local-first storage** — SQLite on your device; works offline.
- **Desktop and web parity** — the same app runs natively (macOS, Windows,
  Linux) and in the browser via Docker.
- **Addons** — install community extensions from a file.
- **Private device sync** — optional E2EE sync between your own devices.
- **Multi-platform** — native desktop installers plus a Docker web image.

---

## 4. Installing and first run

### Desktop (macOS, Windows, Linux)

Download the installer for your platform from GitHub Releases:

```
https://github.com/Sensible-Analytics/folio/releases
```

| Platform | File type             |
| -------- | --------------------- |
| macOS    | `.dmg`                |
| Windows  | `.msi` or NSIS `.exe` |
| Linux    | `.AppImage`           |

Open the downloaded file and follow the standard install steps for your OS. No
configuration is required on first launch.

### Web mode (Docker)

If you prefer browser access, or want to run Folio on a home server:

```bash
docker run -d \
  -p 7472:7472 \
  -v /path/to/your/data:/data \
  -e WF_DB_PATH=/data/folio.db \
  -e WF_SECRET_KEY=your-32-byte-secret-key-here \
  ghcr.io/sensible-analytics/folio:latest
```

Then open `http://localhost:7472`.

> **Note:** Bank Connect is a desktop-only feature. In web mode, use CSV/OFX/QIF
> import instead.

### First run: onboarding

The first time you launch Folio you are guided through an **onboarding** flow.
It introduces the app and helps you create your first account and set up your
preferences. You can re-run or adjust these settings later at any time from
**Settings**.

---

## 5. Configuring Folio

All configuration lives under **Settings** in the sidebar. Below is what each
section controls and how to use it.

### General settings

Basic application preferences: default currency, language/format, and other
startup options. Set your **base currency** first — it drives how values are
displayed across the app.

### Accounts

Create and manage accounts. Each account has a name, a currency, and a type. Use
one account per real-world brokerage, bank, or cash account.

> **Example:** "CommSec Trading", "ANZ Everyday", "Super — AustralianSuper".

### Securities / market data

- **Securities** — the list of assets (tickers/instruments) Folio knows about.
- **Market data** — configure where prices come from, and **import market data**
  from a file if you have offline prices.

Keeping market data current is what makes your valuation and performance numbers
accurate.

### Taxonomies

Classification schemes used to group your assets (for example asset class,
sector, region). Taxonomies power the allocation charts and breakdowns. Adjust
them if you want categories that match how _you_ think about your portfolio.

### Contribution limits

Define contribution caps to track against — useful for superannuation
concessional and non-concessional limits.

### Goals

Create financial goals (for example, a savings target or a target portfolio
value). Goals give you a progress indicator you can monitor over time.

### Appearance

Theme and display preferences.

### AI providers

Configure the AI assistant. Add a provider (and credentials) here to enable the
**Assistant**. If you prefer not to use AI features, leave this empty — the rest
of the app works fully without it.

### Connect (Wealthfolio Connect)

Configure optional **encrypted device sync** so your Folio data appears on your
other devices. Keys stay on your devices; no third-party server can read your
data. Leave this off if you only use Folio on one machine.

### Addons

Install and manage addons (see [§6.10](#610-extend-folio-with-addons)).

### Exports

Export your data — for example, for backup, reporting, or handing to an
accountant.

### About

Version information and app details.

---

## 6. Managing your personal finances

This section walks through the day-to-day workflows, in the order most people
adopt them.

### 6.1 Track your investments

**Where:** Dashboard, **Holdings**, **Holdings Insights**, and individual
**Asset** pages.

- The **Dashboard** (Portfolio page) is your home view: total value, allocation,
  and performance at a glance.
- **Holdings** lists every position you own, per account.
- **Holdings Insights** drills into what you hold across accounts.
- Click any asset to open its **Asset Profile** page for activity and exposure
  in that instrument.

### 6.2 Import your transactions

Everything in Folio is built from **activities**. There are three ways to get
them in:

1. **Manual entry.** Open an account and use **Add Activity**, or go to
   **Activities → Manage** to review and edit what you have. Best for a small
   number of transactions or corrections.

2. **File import (CSV / OFX / QIF).** Export your history from your bank or
   broker, then use the **Import** screen. Folio reads **CSV**, **OFX**, and
   **QIF** files and maps the columns/fields to its activity types. You get to
   review the mapping before it is committed.

3. **Bank Connect (desktop).** Let Folio log into your bank or broker's site and
   pull the transactions for you. See [§6.3](#63-bank-connect-automatic-import).

> **Tip:** Before a large import, skim the
> [activity types reference](./activities/activity-types.md) so you map your
> provider's categories correctly. Unmapped rows become `UNKNOWN` placeholders
> that you can reclassify later.

### 6.3 Bank Connect (automatic import)

**Where:** **Bank Connect** in the sidebar (desktop app only).

Bank Connect is an embedded browser panel inside Folio. It lets you log into
your bank or broker and automatically import your transactions, without ever
handing your credentials to a third-party aggregation service.

How to use it:

1. Open **Bank Connect** from the sidebar.
2. Choose your institution and open the connector.
3. The panel loads the institution's real website in an embedded browser.
4. Log in as you normally would — **on the bank's own site**.
5. Folio detects the transaction history on the page and offers to import it.
6. Review the detected transactions and confirm the import.

Your credentials are entered directly on the bank's website inside the panel.
They are **never stored by Folio and never sent anywhere** — the panel runs
entirely on your local machine.

Connector status is shown as you go: **Idle → Awaiting Login → Logged In →
Downloading… → Complete** (or **Error** if something goes wrong).

Supported institutions are listed in [§8](#8-supported-banks-and-brokers). For
banks that are not yet supported by Bank Connect, use CSV/OFX/QIF import — most
Australian banks let you export a statement file.

### 6.4 Understand your performance

**Where:** **Performance**.

Folio computes standard performance metrics so you can judge how your portfolio
is really doing:

- **TWR (time-weighted return)** — how your investments performed, independent
  of when you added or withdrew money.
- **IRR (money-weighted return)** — the return on the money you actually put in.

Use **Benchmark comparison** on the performance view to line your results up
against a chosen index.

### 6.5 Track income and dividends

**Where:** **Income**.

See the dividends, interest, and distributions you have received, over any
period you choose. Dividend reinvestment (DRIP) is handled so reinvested amounts
are reflected correctly rather than appearing as cash you never received.

### 6.6 See your allocations and insights

**Where:** **Insights**, **Holdings Insights**, allocation charts on the
dashboard.

Allocation charts break your portfolio down by **asset class**, **sector**, and
**currency** (driven by your taxonomies). The **Insights** screens surface
patterns — concentration, drift, and other observations — to help you decide
what to do next.

### 6.7 Track your net worth

**Where:** Dashboard.

Folio brings investment and cash/bank accounts into a single view so you can see
your **net worth** as one number, and track how it changes over time.

### 6.8 Set and track goals

**Where:** **Settings → Goals** (and the goals widget on the dashboard).

Create a goal with a target and a date. Folio shows progress toward it as your
balances change. The bundled **goal-progress-tracker** addon adds a dedicated
progress view (see [§6.10](#610-extend-folio-with-addons)).

### 6.9 Plan for tax

Folio helps you prepare for tax time:

- **Capital gains and income summaries** are derived from your activities
  (`SELL` activities are matched using FIFO cost basis by default).
- **Contribution limits** (Settings) let you watch contributions against caps.
- **Exports** and reports let you pull the numbers you need for your return or
  for your accountant.

> Folio is a tracking tool, not tax advice. Always confirm treatment with a
> qualified professional.

### 6.10 Use the AI assistant

**Where:** **Assistant** in the sidebar (requires an AI provider configured in
**Settings → AI Providers**).

The assistant lets you ask questions about your own data in natural language —
for example, to summarise your holdings, explain a change, or help map a CSV
during import. It runs against your local data and the provider you configured.

### 6.11 Extend Folio with addons

**Where:** **Settings → Addons**.

Addons are optional extensions that add new views or features. They run in a
sandboxed environment and reach your data only through a controlled API.

To install an addon:

1. Download the addon file (`.folio-addon`).
2. In Folio, go to **Settings → Addons**.
3. Click **Install from file** and select the file.
4. The addon's views appear in the sidebar immediately.

To remove one, go to **Settings → Addons**, find it, and click **Uninstall**.

Folio ships with three example addons: **goal-progress-tracker**,
**investment-fees-tracker**, and **swingfolio-addon**. For addon development,
see [./addons/addon-getting-started.md](./addons/addon-getting-started.md).

---

## 7. Activity types reference

Every transaction is one of 14 canonical types. Choosing the right type matters
— each affects your cash balance, holdings, cost basis, and performance
differently.

| Type           | What it represents                                                                                                      |
| -------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `BUY`          | Purchase of a security. Decreases cash, increases holdings.                                                             |
| `SELL`         | Sale of a security. Increases cash, decreases holdings (FIFO cost matching).                                            |
| `SPLIT`        | Stock split or reverse split. Adjusts quantity and per-share cost basis; total cost unchanged.                          |
| `DEPOSIT`      | Cash arriving from outside your portfolio (e.g., funding your brokerage account). Counts as a new capital contribution. |
| `WITHDRAWAL`   | Cash leaving to an external destination. Reduces net contribution.                                                      |
| `TRANSFER_IN`  | Moving cash or assets into this account from another account you own. Cost basis is preserved.                          |
| `TRANSFER_OUT` | Moving cash or assets out of this account to another account you own. Cost basis is preserved.                          |
| `DIVIDEND`     | Cash dividend paid by a holding. Increases cash; does not affect holdings or cost basis.                                |
| `INTEREST`     | Interest earned on cash or fixed-income. Increases cash.                                                                |
| `CREDIT`       | Refund, rebate, or bonus. Increases cash. (A `BONUS` subtype counts as new capital; `REBATE`/`REFUND` do not.)          |
| `FEE`          | Stand-alone fee not attached to a trade (e.g., account management fee). Decreases cash.                                 |
| `TAX`          | Tax paid from the account (e.g., withholding). Decreases cash.                                                          |
| `ADJUSTMENT`   | Non-trade correction — use for corporate actions, option expirations, return-of-capital basis changes.                  |
| `UNKNOWN`      | Placeholder for unrecognized imports. Has no automatic effect until you reclassify it.                                  |

**`TRANSFER_IN` vs `DEPOSIT`.** Use `DEPOSIT` when cash comes from _outside_
your tracked portfolio. Use `TRANSFER_IN` when moving between two accounts you
both track in Folio — that way the move nets to zero at the portfolio level and
does not inflate your net contribution.

**`ADJUSTMENT` vs `UNKNOWN`.** Use `ADJUSTMENT` when you know what the
correction is but no other type fits. Use `UNKNOWN` as a temporary placeholder
for imported rows you have not classified yet.

For the complete reference, including subtypes such as `DRIP` and
`STAKING_REWARD` and their field requirements, see
[./activities/activity-types.md](./activities/activity-types.md).

---

## 8. Supported banks and brokers

### Bank Connect (automatic import)

The following institutions are supported by the built-in Bank Connect panel:

| Institution         | Type   |
| ------------------- | ------ |
| CommBank (CBA)      | Bank   |
| ANZ                 | Bank   |
| ING                 | Bank   |
| Bank of Melbourne   | Bank   |
| Beyond Bank         | Bank   |
| Interactive Brokers | Broker |

**Coming soon** (broker support in progress): CommSec, Sharesies, SelfWealth,
Stake.

### Importing from any other institution

If your bank or broker is not listed above, you can still track it. Most
Australian institutions let you export a statement, and Folio imports **CSV,
OFX, and QIF** files. Use the **Import** screen and map the columns to Folio's
activity types.

---

## 9. Privacy and security

- **Local-first.** Your data is stored in a SQLite database on your device.
- **No cloud storage.** Folio does not store your financial data on any server.
- **No tracking.** No analytics or tracking.
- **Open source.** You can audit the code yourself.
- **Credentials stay put.** Bank Connect enters your login directly on the
  bank's own site inside a local panel; Folio never stores or transmits them.
- **Encrypted sync (optional).** If you enable Wealthfolio Connect, your
  database is replicated between your own devices with end-to-end encryption.
  Keys stay on your devices.

Never commit secrets (API keys, tokens, database credentials) to the repository.
See [SECURITY.md](./SECURITY.md) for the full policy.

---

## 10. Troubleshooting and FAQ

**Where is my data stored?**

In a local SQLite database. In desktop mode it lives in the app's data
directory; in web/Docker mode it is the file at `WF_DB_PATH`.

**Can I use Folio on more than one computer?**

Yes. Enable **Wealthfolio Connect** (Settings → Connect) for encrypted sync
between your own devices, or copy the database file.

**A transaction imported as `UNKNOWN`. What do I do?**

Open **Activities → Manage**, find the transaction, and reclassify it to the
correct type. `UNKNOWN` activities have no effect on your holdings until you do.

**Why is my portfolio value or performance wrong?**

Check that (a) all activities are recorded, (b) market data is current (Settings
→ Market Data), and (c) transaction types are classified correctly.

**Bank Connect isn't available.**

Bank Connect is a desktop-only feature. In web mode, use CSV/OFX/QIF import.

**My bank isn't in the Bank Connect list.**

Export a statement from your bank and import it (CSV/OFX/QIF). Support for more
institutions is being added over time.

**Do I need the AI provider configured?**

No. AI features are optional. Everything else works without a provider.

---

## 11. Getting help

- **Getting Started:** [./getting-started.md](./getting-started.md)
- **Activity types:**
  [./activities/activity-types.md](./activities/activity-types.md)
- **Addons:** [./addons/index.md](./addons/index.md)
- **Security policy:** [./SECURITY.md](./SECURITY.md)
- **Issues and questions:**
  [GitHub Issues](https://github.com/Sensible-Analytics/folio/issues)

---

## Disclaimer

Folio is a personal finance tracking tool and is **not financial advice**.
Always consult a qualified financial advisor before making investment decisions.
