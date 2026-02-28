# Getting Started with Sensible Folio

Sensible Folio is a local-first investment tracker. This guide covers
installation, basic setup, and the core concepts you need to start tracking your
portfolio.

---

## 1. What is Sensible Folio

Sensible Folio tracks your investments — stocks, ETFs, bonds, crypto, and cash
accounts — in one place. There is no cloud service, no subscription, and no
account required. All your data lives in a SQLite database on your own device.

Key characteristics:

- **Local-first.** Data never leaves your machine unless you explicitly set up
  device sync.
- **Multi-platform.** Runs as a native desktop app on macOS, Windows, and Linux.
  Also runs as a web app (via Docker) for browser access on your own server.
- **Extensible.** An addon system lets you install community-built features
  without modifying the core app.
- **Private device sync.** If you want your data on a second device, E2EE
  (end-to-end encrypted) sync lets you replicate it to your own devices — no
  third-party server involved.

---

## 2. Installation

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

If you prefer browser access or want to run Folio on a home server:

```bash
docker run -d \
  -p 7472:7472 \
  -v /path/to/your/data:/data \
  -e WF_DB_PATH=/data/folio.db \
  -e WF_SECRET_KEY=your-32-byte-secret-key-here \
  ghcr.io/sensible-analytics/folio:latest
```

Then open `http://localhost:7472` in your browser.

Required environment variables:

- `WF_DB_PATH` — path to the SQLite database file inside the container
- `WF_SECRET_KEY` — 32-byte secret key used for encryption (generate once, keep
  it safe)

Optional: set `WF_AUTH_PASSWORD_HASH` (Argon2id hash) to require a password on
the web interface.

---

## 3. Getting Started: the General Flow

### Step 1 — Add an account

An account represents a single brokerage, bank, or cash account. Go to
**Accounts** and create one. Give it a name (e.g., "Schwab Taxable") and choose
a currency.

You can create as many accounts as you have real-world accounts.

### Step 2 — Import your activities

An "activity" is any event that changes your account — a trade, a deposit, a
dividend payment, and so on. There are three ways to get activities into Folio:

1. **Manual entry.** Click **Add Activity** inside an account and fill in the
   form. Good for a small number of transactions.

2. **CSV import.** Export a transaction history from your broker and import it
   via **Import → CSV**. Folio maps the CSV columns to its activity types.

3. **Bank Connect.** An embedded browser panel lets you log into your bank or
   broker and auto-import transactions. See [Bank Connect](#5-bank-connect)
   below.

### Step 3 — View your portfolio

Once activities are recorded, go to the **Portfolio** or **Holdings** view. You
will see:

- Current holdings (positions) per account
- Performance metrics (TWR, IRR)
- Allocation charts by asset class, sector, or currency
- Income history (dividends, interest)

---

## 4. Activity Types

Every transaction in Folio is one of 14 canonical activity types. Choosing the
right type matters because each one affects your cash balance, holdings, cost
basis, and performance calculations differently.

### Quick reference

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

### TRANSFER_IN vs DEPOSIT

Use `DEPOSIT` when cash comes from outside your portfolio (a bank transfer
funding your brokerage). Use `TRANSFER_IN` when moving cash or assets between
two accounts that are both tracked in Folio — that way the move nets to zero at
the portfolio level and does not inflate your net contribution figure.

### ADJUSTMENT vs UNKNOWN

Use `ADJUSTMENT` when you know what the correction is but none of the other
types fit (e.g., a spinoff, a basis step-up). Use `UNKNOWN` as a temporary
placeholder for imported transactions you have not yet classified. `UNKNOWN`
activities are flagged for review and have no effect on your holdings until you
reclassify them.

For a complete reference including subtypes (DRIP, STAKING_REWARD, etc.) and
field requirements, see
[./activities/activity-types.md](./activities/activity-types.md).

---

## 5. Bank Connect

Bank Connect is an embedded browser panel built into the desktop app. It lets
you log into your bank or broker's website inside Folio and automatically import
your transactions — without copying credentials to any third-party service.

How it works:

1. Open **Bank Connect** from the sidebar.
2. The panel loads your bank's website in an embedded browser window inside the
   app.
3. Log in as you normally would on the bank's site.
4. Folio detects your transaction history on the page and offers to import it.
5. Review the detected transactions and confirm the import.

Your login credentials are entered directly on the bank's own website, inside
the embedded panel. They are never stored by Folio or sent anywhere — the panel
runs entirely on your local machine.

Bank Connect is most useful for banks and brokers that do not offer a CSV export
or whose CSV format is not yet supported.

---

## 6. Addons

Addons are optional extensions that add new views or features to Folio. They run
inside a sandboxed environment and have access to the same data as the core app
through a controlled API.

### Finding addons

Community addons are distributed via the GitHub releases page and the
repository:

```
https://github.com/Sensible-Analytics/folio/releases
```

Look for files ending in `.folio-addon` or check the `addons/` folder in the
repository.

Examples of what addons can provide:

- Goal tracking (track progress toward a savings or investment target)
- Fee analysis (break down what you are paying in management fees over time)
- Custom charts and allocation views
- Broker-specific import helpers

### Installing an addon

1. Download the addon file (`.folio-addon`).
2. In Folio, go to **Settings → Addons**.
3. Click **Install from file** and select the downloaded file.
4. The addon appears in the sidebar immediately.

To remove an addon, go to **Settings → Addons**, find it in the list, and click
**Uninstall**.

For addon development documentation, see
[./addons/addon-getting-started.md](./addons/addon-getting-started.md).

---

## 7. Design Choices

Understanding why Folio works the way it does helps you use it correctly.

**Local-first.** The SQLite database lives on your device. There is no sync
server, no account to create, and no dependency on an external service being
online. Your portfolio data is available whether or not you have internet
access.

**Desktop and web parity.** The same React frontend runs on both the Tauri
desktop app and the Axum web server. An adapter layer at build time routes
backend calls to either Tauri IPC (desktop) or HTTP (web), so both modes behave
identically.

**Extensible via addons.** The addon system lets the community build features
without forking the app. Addons are isolated: they cannot modify core data
directly and communicate through a defined API.

**Private device sync.** If you want Folio on more than one device, E2EE sync
replicates your database between your own devices. The encryption keys stay on
your devices — no third-party server can read your data.

---

## Next Steps

- Add your first account and record a few activities manually to get a feel for
  the flow.
- Review the [activity types reference](./activities/activity-types.md) before
  doing a large CSV import, so you can map your broker's transaction categories
  correctly.
- Browse the [addons documentation](./addons/index.md) if you want to extend
  what Folio can do.
