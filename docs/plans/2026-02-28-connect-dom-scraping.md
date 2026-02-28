# Connect: DOM-Scraping Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to
> implement this plan task-by-task.

**Goal:** Replace URL-collection bank automation with full DOM transaction
scraping, embedded webview panel, live action log, account auto-detection, and
idempotent activity import.

**Architecture:** A child Tauri webview (not a separate window) is positioned
over a React panel container. Bank automation scripts run inside that webview,
scraping transaction rows directly from the HTML DOM and sending structured JSON
to Rust via Tauri IPC. Rust creates `Account` and `Activity` records with
SHA-256 idempotency keys, deduplicated via SQLite's `ON CONFLICT DO NOTHING`.

**Tech Stack:** Tauri v2 multi-webview, Rust (`sha2` crate for hashing), React +
React Query, shadcn/ui, SQLite/Diesel ORM.

---

## Background: What Already Exists

- `activities.idempotency_key` column + unique index — already in DB ✅
- `activity.source_system`, `source_record_id`, `needs_review`, `import_run_id`
  — all exist ✅
- `BankKey` enum, `BankConnectSettings`, `BankDownloadRun` model — exist ✅
- `bank_download_runs` SQLite table — exists ✅
- Frontend bank-connect page with Banks/Brokers sections + log panel — exists ✅
- Bank automation scripts (ING/CBA/ANZ/BOM/Beyond/IBKR) — exist but only collect
  URLs ✅

**What is missing:** DOM scraping in scripts → IPC → Rust import pipeline →
embedded webview.

---

## Phase 1: New IPC Data Models

### Task 1: Add ScrapedTransaction and ScrapedAccount models to core

**Files:**

- Modify: `crates/core/src/bank_connect/models.rs`

**Step 1: Add structs**

```rust
/// A single transaction row scraped from bank DOM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapedTransaction {
    pub date: String,             // "2024-01-15" (YYYY-MM-DD)
    pub description: String,
    pub amount: f64,              // negative = debit, positive = credit
    pub balance: Option<f64>,
    pub reference: Option<String>,// bank's own transaction ID if available
    pub transaction_type: Option<String>, // "DEBIT", "CREDIT", "TRANSFER"
}

/// Account metadata scraped from bank DOM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapedAccount {
    pub bank_key: String,
    pub account_name: String,     // "Orange Everyday"
    pub account_number: String,   // "1234 5678" (display format)
    pub bsb: Option<String>,      // "923-100"
    pub currency: String,         // "AUD"
    pub account_type: String,     // "TRANSACTION", "SAVINGS", "INVESTMENT"
    pub current_balance: Option<f64>,
}

/// Batch of transactions for one account, sent from JS via IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapedTransactionBatch {
    pub bank_key: String,
    pub run_id: String,
    pub account: ScrapedAccount,
    pub transactions: Vec<ScrapedTransaction>,
}

/// Agent step event for the live log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStepEvent {
    pub bank_key: String,
    pub step: String,             // "navigate", "scrape", "import", "done", "error"
    pub message: String,
    pub detail: Option<String>,   // extra context
    pub timestamp: String,        // ISO 8601
}
```

**Step 2: Add `sha2` to crates/core/Cargo.toml**

```toml
sha2 = "0.10"
```

**Step 3: Add idempotency key function**

```rust
use sha2::{Digest, Sha256};

pub fn make_idempotency_key(
    bank_key: &str,
    account_number: &str,
    tx: &ScrapedTransaction,
) -> String {
    let input = format!(
        "BANK_CONNECT|{}|{}|{}|{:.2}|{}",
        bank_key,
        account_number,
        tx.date,
        tx.amount,
        tx.description.trim(),
    );
    format!("{:x}", Sha256::digest(input.as_bytes()))
}
```

**Step 4: Write unit test**

In `crates/core/src/bank_connect/models.rs` (or a test module at bottom):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_key_is_deterministic() {
        let tx = ScrapedTransaction {
            date: "2024-01-15".into(),
            description: "WOOLWORTHS 0123".into(),
            amount: -42.50,
            balance: Some(1234.56),
            reference: None,
            transaction_type: Some("DEBIT".into()),
        };
        let k1 = make_idempotency_key("ING", "12345678", &tx);
        let k2 = make_idempotency_key("ING", "12345678", &tx);
        assert_eq!(k1, k2);
        assert_eq!(k1.len(), 64); // SHA-256 hex
    }

    #[test]
    fn idempotency_key_differs_by_bank() {
        let tx = ScrapedTransaction {
            date: "2024-01-15".into(),
            description: "WOOLWORTHS".into(),
            amount: -42.50,
            balance: None,
            reference: None,
            transaction_type: None,
        };
        let k_ing = make_idempotency_key("ING", "12345678", &tx);
        let k_cba = make_idempotency_key("CBA", "12345678", &tx);
        assert_ne!(k_ing, k_cba);
    }
}
```

**Step 5: Run tests**

```bash
cargo test -p sensible-folio-core bank_connect
```

Expected: 2 tests PASS.

**Step 6: Commit**

```bash
git add crates/core/src/bank_connect/models.rs crates/core/Cargo.toml
git commit -m "feat(bank-connect): add ScrapedTransaction/Account models and idempotency key"
```

---

## Phase 2: Import Pipeline (Rust)

### Task 2: Detect/create Account from ScrapedAccount

**Files:**

- Modify: `apps/tauri/src/commands/bank_connect.rs`
- Reference: `crates/core/src/accounts/` (find `create_account` or equivalent
  service fn)

**Step 1: Find account service function**

Read `crates/core/src/accounts/accounts_service.rs` — find
`create_account(NewAccount)` signature.

**Step 2: Write failing test** (integration test, skip if account service
requires DB)

```rust
// In apps/tauri/src/commands/bank_connect.rs — add unit test at bottom
#[cfg(test)]
mod tests {
    use sensible_folio_core::bank_connect::models::ScrapedAccount;

    fn sample_scraped_account() -> ScrapedAccount {
        ScrapedAccount {
            bank_key: "ING".into(),
            account_name: "Orange Everyday".into(),
            account_number: "12345678".into(),
            bsb: Some("923-100".into()),
            currency: "AUD".into(),
            account_type: "TRANSACTION".into(),
            current_balance: Some(1234.56),
        }
    }

    #[test]
    fn scraped_account_to_new_account_mapping() {
        let scraped = sample_scraped_account();
        // provider_account_id = "{bank_key}:{account_number}"
        let provider_id = format!("{}:{}", scraped.bank_key, scraped.account_number);
        assert_eq!(provider_id, "ING:12345678");
    }
}
```

**Step 3: Add `find_or_create_bank_account` helper**

In `apps/tauri/src/commands/bank_connect.rs`:

```rust
async fn find_or_create_bank_account(
    ctx: &ServiceContext,
    scraped: &ScrapedAccount,
) -> Result<String, String> {
    let provider_id = format!("{}:{}", scraped.bank_key, scraped.account_number);

    // Check if account already exists by provider_account_id
    let accounts = ctx
        .account_service
        .get_accounts()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(existing) = accounts
        .iter()
        .find(|a| a.provider_account_id.as_deref() == Some(&provider_id))
    {
        return Ok(existing.id.clone());
    }

    // Create new account
    let new_account = NewAccount {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("{} - {}", scraped.bank_key, scraped.account_name),
        account_type: map_account_type(&scraped.account_type),
        group: None,
        currency: scraped.currency.clone(),
        is_default: false,
        is_active: true,
        platform_id: None,
        account_number: Some(scraped.account_number.clone()),
        meta: None,
        provider: Some("BANK_CONNECT".to_string()),
        provider_account_id: Some(provider_id),
        tracking_mode: TrackingMode::Transactions,
        is_archived: false,
    };

    ctx.account_service
        .create_account(new_account)
        .await
        .map(|a| a.id)
        .map_err(|e| e.to_string())
}

fn map_account_type(raw: &str) -> String {
    match raw.to_uppercase().as_str() {
        "SAVINGS" => "SAVINGS".to_string(),
        "INVESTMENT" => "TRADING".to_string(),
        _ => "CHECKING".to_string(),
    }
}
```

**Step 4: Verify it compiles**

```bash
cargo check -p sensible-folio-app
```

**Step 5: Commit**

```bash
git add apps/tauri/src/commands/bank_connect.rs
git commit -m "feat(bank-connect): add find_or_create_bank_account helper"
```

---

### Task 3: Import transaction batch as Activities

**Files:**

- Modify: `apps/tauri/src/commands/bank_connect.rs`

**Step 1: Add `import_scraped_transactions` Tauri command**

```rust
#[tauri::command]
pub async fn import_scraped_transactions(
    batch: ScrapedTransactionBatch,
    state: State<'_, Arc<ServiceContext>>,
    app: AppHandle,
) -> Result<ImportResult, String> {
    let ctx = state.inner();
    let account_id = find_or_create_bank_account(ctx, &batch.account).await?;

    let mut new_count = 0u32;
    let mut skipped_count = 0u32;

    for tx in &batch.transactions {
        let ikey = make_idempotency_key(&batch.bank_key, &batch.account.account_number, tx);

        // Parse date
        let date = chrono::NaiveDate::parse_from_str(&tx.date, "%Y-%m-%d")
            .map_err(|e| format!("Bad date '{}': {}", tx.date, e))?;
        let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();

        // Determine activity type
        let (activity_type, quantity, unit_price) = if tx.amount < 0.0 {
            ("WITHDRAWAL", Some(Decimal::from_f64(tx.amount.abs()).unwrap()), Some(Decimal::ONE))
        } else {
            ("DEPOSIT", Some(Decimal::from_f64(tx.amount).unwrap()), Some(Decimal::ONE))
        };

        let new_activity = NewActivity {
            id: uuid::Uuid::new_v4().to_string(),
            account_id: account_id.clone(),
            asset_id: None,
            activity_type: activity_type.to_string(),
            source_type: None,
            subtype: None,
            status: Some(ActivityStatus::Posted),
            activity_date: datetime,
            settlement_date: None,
            quantity: quantity,
            unit_price: unit_price,
            amount: Some(Decimal::from_f64(tx.amount.abs()).unwrap()),
            fee: None,
            currency: batch.account.currency.clone(),
            fx_rate: None,
            notes: Some(tx.description.clone()),
            metadata: None,
            source_system: Some("BANK_CONNECT".to_string()),
            source_record_id: tx.reference.clone(),
            source_group_id: None,
            idempotency_key: Some(ikey),
            import_run_id: Some(batch.run_id.clone()),
            needs_review: Some(false),
        };

        match ctx.activity_service.create_activity(new_activity).await {
            Ok(_) => new_count += 1,
            Err(e) if e.to_string().contains("UNIQUE constraint") => {
                skipped_count += 1;
            }
            Err(e) => return Err(e.to_string()),
        }
    }

    // Update run record
    ctx.bank_connect_repo
        .update_run_stats(&batch.run_id, new_count as i32, skipped_count as i32)
        .await
        .map_err(|e| e.to_string())?;

    // Emit event so frontend log updates
    let _ = app.emit(
        "bank://import-complete",
        serde_json::json!({
            "bankKey": batch.bank_key,
            "runId": batch.run_id,
            "accountId": account_id,
            "newCount": new_count,
            "skippedCount": skipped_count,
        }),
    );

    Ok(ImportResult { account_id, new_count, skipped_count })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub account_id: String,
    pub new_count: u32,
    pub skipped_count: u32,
}
```

**Step 2: Add `update_run_stats` to bank_connect repository**

In `crates/storage-sqlite/src/bank_connect/repository.rs`, add:

```rust
pub async fn update_run_stats(
    &self,
    run_id: &str,
    files_downloaded: i32,
    files_skipped: i32,
) -> Result<(), Error> {
    diesel::update(bank_download_runs::table.find(run_id))
        .set((
            bank_download_runs::files_downloaded.eq(files_downloaded),
            bank_download_runs::files_skipped.eq(files_skipped),
            bank_download_runs::status.eq("completed"),
            bank_download_runs::completed_at.eq(chrono::Utc::now().to_rfc3339()),
        ))
        .execute(&mut self.pool.get()?)?;
    Ok(())
}
```

**Step 3: Wire command into lib.rs**

In `apps/tauri/src/lib.rs`, add `import_scraped_transactions` to the
`invoke_handler!` macro.

**Step 4: Compile check**

```bash
cargo check -p sensible-folio-app
```

**Step 5: Commit**

```bash
git add apps/tauri/src/commands/bank_connect.rs \
        crates/storage-sqlite/src/bank_connect/repository.rs \
        apps/tauri/src/lib.rs
git commit -m "feat(bank-connect): add import_scraped_transactions command with idempotency"
```

---

## Phase 3: Rewrite Automation Scripts (DOM Scraping)

> Each bank script follows the same pattern: navigate → scrape account list →
> for each account scrape transaction rows → send batch via IPC. Replace
> `bank_urls` IPC call with `bank_transactions`.

### Task 4: ING script — DOM transaction scraping

**Files:**

- Modify: `apps/tauri/src/banks/ing.rs`

**Step 1: Replace ING_SCRIPT with DOM scraping version**

Key selectors for ING (from connect-instruction.md):

- Account list: `select[id*='account'] option` or `.account-selector option`
- Transaction rows: `table.transactions tbody tr`, `.transaction-list li`
- Date cell: `td:nth-child(1)`, `[class*='date']`
- Description cell: `td:nth-child(2)`, `[class*='description']`
- Amount cell: `td:nth-child(3)`, `[class*='amount']`
- Balance cell: `td:nth-child(4)`, `[class*='balance']`

```rust
pub const ING_SCRIPT: &str = r#"
(async function ingDomScraper() {
  const BANK_KEY = 'ING';
  const YEARS_BACK = __YEARS_BACK__;
  const cutoffDate = new Date();
  cutoffDate.setFullYear(cutoffDate.getFullYear() - YEARS_BACK);

  function log(level, message, detail) {
    window.__TAURI_INTERNALS__.ipc.postMessage(JSON.stringify({
      cmd: 'bank_progress',
      payload: { bankKey: BANK_KEY, level, message, detail: detail || null,
                 timestamp: new Date().toISOString() }
    }));
  }

  function sendBatch(runId, account, transactions) {
    window.__TAURI_INTERNALS__.ipc.postMessage(JSON.stringify({
      cmd: 'bank_transactions',
      payload: { bankKey: BANK_KEY, runId, account, transactions }
    }));
  }

  function parseAmount(text) {
    const clean = text.replace(/[^0-9.,-]/g, '').replace(',', '');
    const num = parseFloat(clean);
    // ING shows debits with '-' prefix or in red column
    return isNaN(num) ? 0 : num;
  }

  function parseDate(text) {
    // ING shows "15 Jan 2024" or "2024-01-15"
    const isoMatch = text.match(/(\d{4})-(\d{2})-(\d{2})/);
    if (isoMatch) return isoMatch[0];
    const dmyMatch = text.match(/(\d{1,2})\s+(\w{3})\s+(\d{4})/);
    if (dmyMatch) {
      const months = {Jan:'01',Feb:'02',Mar:'03',Apr:'04',May:'05',Jun:'06',
                      Jul:'07',Aug:'08',Sep:'09',Oct:'10',Nov:'11',Dec:'12'};
      return `${dmyMatch[3]}-${months[dmyMatch[2]]}-${dmyMatch[1].padStart(2,'0')}`;
    }
    return null;
  }

  const runId = '__RUN_ID__';

  try {
    log('info', 'Navigating to accounts...');
    // Navigate to transaction history
    const txLink = document.querySelector("a[href*='transaction'], a[href*='history'], nav a[href*='account']");
    if (txLink) { txLink.click(); await new Promise(r => setTimeout(r, 2000)); }

    // Find account selector
    const accountSelect = document.querySelector("select[id*='account'], select[name*='account'], .account-picker select");
    const accountOptions = accountSelect
      ? Array.from(accountSelect.options).filter(o => o.value)
      : [null]; // single account, no selector

    log('info', `Found ${accountOptions.length} account(s)`);

    for (const opt of accountOptions) {
      if (opt && accountSelect) {
        accountSelect.value = opt.value;
        accountSelect.dispatchEvent(new Event('change', { bubbles: true }));
        await new Promise(r => setTimeout(r, 1500));
      }

      // Scrape account meta from page header
      const accountName = document.querySelector('.account-name, h1, .account-header')?.textContent?.trim()
        || opt?.text?.trim() || 'ING Account';
      const accountNumber = document.querySelector('.account-number, [class*="accountNumber"]')?.textContent?.trim()
        || opt?.value?.trim() || '0000';
      const balanceText = document.querySelector('.balance, [class*="balance"]')?.textContent || '0';
      const balance = parseAmount(balanceText);

      const account = {
        bankKey: BANK_KEY,
        accountName,
        accountNumber: accountNumber.replace(/\s/g, ''),
        bsb: document.querySelector('.bsb, [class*="bsb"]')?.textContent?.trim() || null,
        currency: 'AUD',
        accountType: 'TRANSACTION',
        currentBalance: balance,
      };

      log('info', `Scraping transactions for ${accountName}...`);

      // Collect all transaction rows
      const rows = Array.from(document.querySelectorAll(
        'table tbody tr, .transaction-list li, [class*="transactionRow"], [class*="transaction-item"]'
      ));

      const transactions = [];
      for (const row of rows) {
        const cells = row.querySelectorAll('td, [class*="cell"]');
        if (cells.length < 2) continue;

        const dateText = (cells[0]?.textContent || row.querySelector('[class*="date"]')?.textContent || '').trim();
        const date = parseDate(dateText);
        if (!date) continue;

        if (new Date(date) < cutoffDate) continue; // past cutoff

        const description = (cells[1]?.textContent || row.querySelector('[class*="desc"]')?.textContent || '').trim();
        const amountText = (cells[2]?.textContent || row.querySelector('[class*="amount"]')?.textContent || '0').trim();
        const balText = (cells[3]?.textContent || row.querySelector('[class*="balance"]')?.textContent || '').trim();
        const refText = row.querySelector('[class*="ref"], [class*="id"]')?.textContent?.trim() || null;

        const amount = parseAmount(amountText);
        const bal = balText ? parseAmount(balText) : null;

        transactions.push({ date, description, amount, balance: bal, reference: refText, transactionType: amount < 0 ? 'DEBIT' : 'CREDIT' });
      }

      log('info', `Found ${transactions.length} transactions for ${accountName}`);
      sendBatch(runId, account, transactions);
      log('success', `Sent ${transactions.length} transactions for ${accountName}`);

      await new Promise(r => setTimeout(r, 500));
    }

    log('success', 'ING scraping complete');
  } catch (err) {
    log('error', `ING scraper error: ${err.message}`);
  }
})();
"#;
```

**Step 2: Verify Rust still compiles (scripts are just string constants)**

```bash
cargo check -p sensible-folio-app
```

**Step 3: Commit**

```bash
git add apps/tauri/src/banks/ing.rs
git commit -m "feat(bank-connect): rewrite ING script to scrape DOM transactions"
```

---

### Task 5: CBA script — DOM scraping

Same structure as Task 4, adapted for CBA NetBank selectors:

- Transaction rows: `.transaction-list__item`, `[class*='StatementRow']`,
  `table.transactions tbody tr`
- Date: `[class*='date']`, `td:first-child`
- Description: `[class*='description']`, `.merchant-name`
- Amount: `[class*='amount']`, `[class*='debit']`, `[class*='credit']`

Follow identical pattern as Task 4.

**Commit:**
`"feat(bank-connect): rewrite CBA script to scrape DOM transactions"`

---

### Task 6: ANZ script — DOM scraping

ANZ-specific: check for ANZ Plus (`/plus` URL → log warning, exit). ANZ uses
`[class*='transaction']` rows with separate debit/credit columns.

Follow identical pattern. **Commit:**
`"feat(bank-connect): rewrite ANZ script to scrape DOM transactions"`

---

### Task 7: BOM script — DOM scraping

BOM (Westpac Group portal) — iterate account tabs, scrape
`table.statements tbody tr`.

Follow identical pattern. **Commit:**
`"feat(bank-connect): rewrite BOM script to scrape DOM transactions"`

---

### Task 8: Beyond Bank script — DOM scraping

Beyond Bank — set date range inputs to `today - yearsBack`, click Search,
paginate through pages.

Follow identical pattern. **Commit:**
`"feat(bank-connect): rewrite Beyond script to scrape DOM transactions"`

---

### Task 9: IBKR script — DOM scraping

IBKR — navigate Reports → Activity, scrape trade/cash rows. Note: IBKR has
complex multi-asset rows; map to activity types (BUY/SELL/DIVIDEND/DEPOSIT).

Follow identical pattern. **Commit:**
`"feat(bank-connect): rewrite IBKR script to scrape DOM transactions"`

---

### Task 10: Wire `bank_transactions` IPC in Rust

Currently `bank_progress` and `bank_urls` IPC messages are handled. Add
`bank_transactions` handler.

**Files:**

- Modify: `apps/tauri/src/commands/bank_connect.rs`

**Step 1: Find where `bank_urls` IPC is handled**

Search for the IPC message handler that receives `cmd: 'bank_urls'` or
`bank_progress`. In Tauri v2 this is likely via `window_event` or a
`ipc_handler` registration.

**Step 2: Add handler for `bank_transactions`**

In the IPC message dispatch, add:

```rust
"bank_transactions" => {
    let batch: ScrapedTransactionBatch = serde_json::from_value(payload)
        .map_err(|e| format!("Failed to parse batch: {}", e))?;
    // Spawn async task to avoid blocking IPC thread
    let ctx = state.clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = import_scraped_transactions_inner(batch, &ctx, &app_clone).await {
            let _ = app_clone.emit("bank://progress", AgentStepEvent {
                bank_key: "UNKNOWN".into(),
                step: "error".into(),
                message: format!("Import failed: {}", e),
                detail: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
    });
}
```

**Step 3: Compile + test**

```bash
cargo check -p sensible-folio-app
```

**Commit:**
`"feat(bank-connect): wire bank_transactions IPC to import pipeline"`

---

## Phase 4: Embedded Webview (Tauri Multi-Webview)

### Task 11: Replace separate window with child webview

**Files:**

- Modify: `apps/tauri/src/commands/bank_connect.rs`

**Background:** Tauri v2 supports adding a child `Webview` to an existing
`WebviewWindow`. The child webview renders at a specific pixel rect inside the
parent window. React tells Rust the panel bounds via a new IPC command.

**Step 1: Add `set_bank_webview_bounds` command**

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[tauri::command]
pub async fn open_bank_panel(
    bank_key: String,
    bounds: WebviewBounds,
    app: AppHandle,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<(), String> {
    let parsed_key: BankKey = bank_key.parse().map_err(|e: String| e)?;
    let label = format!("bank-{}", bank_key.to_lowercase());
    let login_url = parsed_key.login_url();
    let post_login = parsed_key.post_login_pattern().to_string();
    let bank_key_clone = bank_key.clone();

    // Remove existing child webview if open
    if let Some(main_window) = app.get_webview_window("main") {
        // Try to destroy existing bank webview
        let _ = app.get_webview(&label).map(|w| w.close());
    }

    let main_window = app.get_webview_window("main")
        .ok_or("Main window not found")?;

    let url = tauri::WebviewUrl::External(
        tauri::Url::parse(login_url).map_err(|e| e.to_string())?
    );

    let app_for_nav = app.clone();
    let bank_key_nav = bank_key.clone();

    // Create child webview inside main window
    main_window
        .add_child(
            tauri::webview::WebviewBuilder::new(&label, url)
                .on_navigation(move |nav_url| {
                    if nav_url.as_str().contains(&post_login) {
                        let _ = app_for_nav.emit(
                            "bank://login-detected",
                            BankLoginDetectedPayload { bank_key: bank_key_nav.clone() },
                        );
                    }
                    true
                }),
            tauri::LogicalPosition::new(bounds.x, bounds.y),
            tauri::LogicalSize::new(bounds.width, bounds.height),
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn close_bank_panel(bank_key: String, app: AppHandle) -> Result<(), String> {
    let label = format!("bank-{}", bank_key.to_lowercase());
    if let Some(wv) = app.get_webview(&label) {
        wv.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn resize_bank_panel(
    bank_key: String,
    bounds: WebviewBounds,
    app: AppHandle,
) -> Result<(), String> {
    let label = format!("bank-{}", bank_key.to_lowercase());
    if let Some(wv) = app.get_webview(&label) {
        wv.set_bounds(tauri::Rect {
            position: tauri::LogicalPosition::new(bounds.x, bounds.y).into(),
            size: tauri::LogicalSize::new(bounds.width, bounds.height).into(),
        }).map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

**Step 2: Wire commands in lib.rs**

Add `open_bank_panel`, `close_bank_panel`, `resize_bank_panel` to
`invoke_handler!`.

**Step 3: Compile check**

```bash
cargo check -p sensible-folio-app
```

**Step 4: Commit**

```bash
git add apps/tauri/src/commands/bank_connect.rs apps/tauri/src/lib.rs
git commit -m "feat(bank-connect): add open/close/resize_bank_panel child webview commands"
```

---

## Phase 5: Frontend Redesign

### Task 12: Update TypeScript adapters

**Files:**

- Modify: `apps/frontend/src/adapters/tauri/bank-connect.ts`

**Step 1: Add new commands**

```typescript
export interface WebviewBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ImportResult {
  accountId: string;
  newCount: number;
  skippedCount: number;
}

export const openBankPanel = async (
  bankKey: string,
  bounds: WebviewBounds,
): Promise<void> => invoke<void>("open_bank_panel", { bankKey, bounds });

export const closeBankPanel = async (bankKey: string): Promise<void> =>
  invoke<void>("close_bank_panel", { bankKey });

export const resizeBankPanel = async (
  bankKey: string,
  bounds: WebviewBounds,
): Promise<void> => invoke<void>("resize_bank_panel", { bankKey, bounds });

export const listenBankImportComplete = (
  handler: (event: {
    payload: ImportResult & { bankKey: string; runId: string };
  }) => void,
) =>
  listen<ImportResult & { bankKey: string; runId: string }>(
    "bank://import-complete",
    handler,
  );
```

**Step 2: Export from adapters/index.ts**

Add to `apps/frontend/src/adapters/index.ts` re-exports.

**Step 3: Type check**

```bash
pnpm type-check
```

**Step 4: Commit**

```bash
git add apps/frontend/src/adapters/tauri/bank-connect.ts apps/frontend/src/adapters/index.ts
git commit -m "feat(bank-connect): add panel webview adapter commands"
```

---

### Task 13: Redesign bank-connect page with embedded panel

**Files:**

- Modify: `apps/frontend/src/pages/bank-connect/bank-connect-page.tsx`

**Layout:**

```
┌─────────────────────────────────────────────────────────┐
│ Connect                                   [Settings ⚙]  │
├──────────────┬──────────────────────────────────────────┤
│ Banks        │                                          │
│ ┌──────────┐ │    [Bank webview panel renders here]     │
│ │ ING  ●  │ │    (Tauri child webview, not React)       │
│ │ CBA     │ │                                          │
│ │ ANZ     │ │    When no bank selected:                │
│ │ BOM     │ │    "Select a bank from the left          │
│ │ BEYOND  │ │     to begin"                            │
│ └──────────┘ │                                          │
│              ├──────────────────────────────────────────┤
│ Brokers      │ Activity Log              [Cancel] [Clear]│
│ ┌──────────┐ │ 10:42:01 [ING] [info]  Navigating...    │
│ │ IBKR β  │ │ 10:42:03 [ING] [✓]     Found 3 accounts │
│ │ CommSec │ │ 10:42:05 [ING] [info]  Scraping #1234... │
│ │ (soon)  │ │ 10:42:07 [ING] [✓]     847 tx imported   │
│ └──────────┘ │                                          │
└──────────────┴──────────────────────────────────────────┘
```

**Key implementation points:**

1. Left panel: connector list (unchanged, already built)
2. Right panel: `<div ref={panelRef} id="bank-webview-container">` — placeholder
   div whose bounds are measured and sent to Rust
3. On connector click: measure `panelRef.current.getBoundingClientRect()`, call
   `openBankPanel(key, bounds)`
4. `ResizeObserver` on `panelRef` → call `resizeBankPanel` when container size
   changes
5. Bottom strip: log panel (already built, keep as-is)
6. **Cancel button**: calls `closeBankPanel(activeBank)` — closes the webview
   mid-session
7. When `bank://login-detected` event fires → auto-inject automation script (no
   manual "Start Download" button)
8. When `bank://import-complete` fires → show toast "X new transactions
   imported"

**Step 1: Replace manual `startBankDownload` with auto-start on login**

In the `listenBankLoginDetected` handler:

```typescript
listenBankLoginDetected((event) => {
  const key = event.payload.bankKey;
  setStatuses((prev) => ({ ...prev, [key]: "logged-in" }));
  addLog(key, "success", "Login detected — starting automation...");
  // Auto-start: inject script immediately
  startBankDownload(key).catch((err) =>
    addLog(key, "error", `Automation failed: ${String(err)}`),
  );
});
```

**Step 2: Add panel ref and ResizeObserver**

```typescript
const panelRef = useRef<HTMLDivElement>(null);
const activeBankRef = useRef<string | null>(null);

useEffect(() => {
  if (!panelRef.current) return;
  const observer = new ResizeObserver(() => {
    if (!activeBankRef.current || !panelRef.current) return;
    const r = panelRef.current.getBoundingClientRect();
    resizeBankPanel(activeBankRef.current, {
      x: r.left,
      y: r.top,
      width: r.width,
      height: r.height,
    }).catch(console.error);
  });
  observer.observe(panelRef.current);
  return () => observer.disconnect();
}, []);
```

**Step 3: Update handleOpenLogin**

```typescript
const handleOpenLogin = async (key: string) => {
  if (!panelRef.current) return;
  const r = panelRef.current.getBoundingClientRect();
  activeBankRef.current = key;
  try {
    await openBankPanel(key, {
      x: r.left,
      y: r.top,
      width: r.width,
      height: r.height,
    });
    setStatuses((prev) => ({ ...prev, [key]: "window-open" }));
    addLog(key, "info", `Opening ${key} — please log in...`);
  } catch (err) {
    addLog(key, "error", `Failed to open panel: ${String(err)}`);
  }
};
```

**Step 4: Add Cancel handler**

```typescript
const handleCancel = async (key: string) => {
  await closeBankPanel(key).catch(console.error);
  activeBankRef.current = null;
  setStatuses((prev) => ({ ...prev, [key]: "idle" }));
  addLog(key, "warn", `${key} session cancelled by user`);
};
```

**Step 5: Type check**

```bash
pnpm type-check
```

**Step 6: Commit**

```bash
git add apps/frontend/src/pages/bank-connect/bank-connect-page.tsx
git commit -m "feat(bank-connect): embedded panel layout with auto-start on login and cancel"
```

---

## Phase 6: New Account Confirmation Modal

### Task 14: "New accounts found" modal

When a new account is auto-created, the user should be notified and given a
chance to rename or reject it.

**Files:**

- Create: `apps/frontend/src/pages/bank-connect/new-accounts-modal.tsx`
- Modify: `apps/frontend/src/pages/bank-connect/bank-connect-page.tsx`

**Step 1: Add `bank://new-account-created` event to Rust**

In `find_or_create_bank_account`, when a NEW account is created, emit:

```rust
let _ = app.emit("bank://new-account-created", serde_json::json!({
    "accountId": new_id,
    "accountName": new_account.name,
    "bankKey": scraped.bank_key,
    "accountNumber": scraped.account_number,
}));
```

**Step 2: Create minimal modal component**

```typescript
// apps/frontend/src/pages/bank-connect/new-accounts-modal.tsx
interface NewAccountInfo {
  accountId: string;
  accountName: string;
  bankKey: string;
  accountNumber: string;
}

export function NewAccountsModal({
  accounts,
  onDismiss,
}: {
  accounts: NewAccountInfo[];
  onDismiss: () => void;
}) {
  if (accounts.length === 0) return null;
  return (
    <Dialog open onOpenChange={onDismiss}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New accounts detected</DialogTitle>
          <DialogDescription>
            The following accounts were found and added automatically.
          </DialogDescription>
        </DialogHeader>
        <ul className="space-y-2 py-2">
          {accounts.map((a) => (
            <li key={a.accountId} className="flex items-center gap-2 text-sm">
              <Building2 className="h-4 w-4 text-muted-foreground" />
              <span className="font-medium">{a.accountName}</span>
              <span className="text-muted-foreground font-mono text-xs">
                •••{a.accountNumber.slice(-4)}
              </span>
            </li>
          ))}
        </ul>
        <DialogFooter>
          <Button onClick={onDismiss}>Got it</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

**Step 3: Wire in bank-connect-page**

Listen for `bank://new-account-created`, accumulate new accounts, show modal.

**Step 4: Type check + commit**

```bash
pnpm type-check
git add apps/frontend/src/pages/bank-connect/
git commit -m "feat(bank-connect): add new account detection modal"
```

---

## Phase 7: Validation

### Task 15: End-to-end compile + tests

**Step 1: Rust compile check**

```bash
cargo check --workspace
```

Expected: no errors.

**Step 2: Run Rust tests**

```bash
cargo test -p sensible-folio-core bank_connect
```

Expected: idempotency key tests pass.

**Step 3: TypeScript type check**

```bash
pnpm type-check
```

Expected: no errors.

**Step 4: Run TS tests**

```bash
pnpm test
```

Expected: no regressions.

**Step 5: Manual smoke test**

1. `pnpm tauri dev`
2. Navigate to Connect page
3. Click ING → verify embedded webview appears in right panel
4. Log in → verify log strip shows "Login detected — starting automation..."
5. Verify log shows navigation steps
6. Verify transactions appear in Activities page after completion
7. Run ING again → verify skipped count equals previous new count (idempotency)

**Step 6: Commit final state**

```bash
git add -A
git commit -m "feat(bank-connect): complete DOM scraping pipeline end-to-end"
```

---

## Summary: Files Changed

| File                                                          | Change                                                                     |
| ------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `crates/core/src/bank_connect/models.rs`                      | Add ScrapedTransaction, ScrapedAccount, AgentStepEvent, idempotency key fn |
| `crates/core/Cargo.toml`                                      | Add sha2 dependency                                                        |
| `crates/storage-sqlite/src/bank_connect/repository.rs`        | Add update_run_stats                                                       |
| `apps/tauri/src/commands/bank_connect.rs`                     | Add import pipeline, panel webview commands                                |
| `apps/tauri/src/lib.rs`                                       | Wire new commands                                                          |
| `apps/tauri/src/banks/ing.rs`                                 | DOM scraping rewrite                                                       |
| `apps/tauri/src/banks/cba.rs`                                 | DOM scraping rewrite                                                       |
| `apps/tauri/src/banks/anz.rs`                                 | DOM scraping rewrite                                                       |
| `apps/tauri/src/banks/bom.rs`                                 | DOM scraping rewrite                                                       |
| `apps/tauri/src/banks/beyond.rs`                              | DOM scraping rewrite                                                       |
| `apps/tauri/src/banks/ibkr.rs`                                | DOM scraping rewrite                                                       |
| `apps/frontend/src/adapters/tauri/bank-connect.ts`            | Add panel commands + import event                                          |
| `apps/frontend/src/adapters/index.ts`                         | Re-export new commands                                                     |
| `apps/frontend/src/pages/bank-connect/bank-connect-page.tsx`  | Embedded panel, auto-start, cancel                                         |
| `apps/frontend/src/pages/bank-connect/new-accounts-modal.tsx` | New file — account notification modal                                      |

**No new migrations needed** — idempotency_key and all required Activity fields
already exist.
