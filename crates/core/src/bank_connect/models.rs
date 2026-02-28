use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// All connectors (banks + brokers) share this key type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BankKey {
    // ── Australian banks ────────────────────────────────────────
    Ing,
    Cba,
    Anz,
    Bom,
    Beyond,
    // ── Brokers / portfolio connectors ──────────────────────────
    Ibkr,
}

impl BankKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            BankKey::Ing => "ING",
            BankKey::Cba => "CBA",
            BankKey::Anz => "ANZ",
            BankKey::Bom => "BOM",
            BankKey::Beyond => "BEYOND",
            BankKey::Ibkr => "IBKR",
        }
    }

    pub fn login_url(&self) -> &'static str {
        match self {
            BankKey::Ing => "https://www.ing.com.au/securebanking/",
            BankKey::Cba => "https://www.netbank.com.au/",
            BankKey::Anz => "https://www.anz.com.au/IBAU/Bank/",
            BankKey::Bom => "https://ibanking.bankofmelbourne.com.au/ibank/loginPage.action",
            BankKey::Beyond => "https://online.beyondbank.com.au/web/banking",
            BankKey::Ibkr => "https://www.ibkr.com.au/sso/Login?RL=1&locale=en_AU",
        }
    }

    pub fn post_login_pattern(&self) -> &'static str {
        match self {
            BankKey::Ing => "securebanking",
            BankKey::Cba => "netbank.com.au/netbank",
            BankKey::Anz => "anz.com.au/IBAU/Bank",
            BankKey::Bom => "ibanking.bankofmelbourne.com.au/ibank",
            BankKey::Beyond => "online.beyondbank.com.au/web/banking#/",
            BankKey::Ibkr => "portal.ibkr.com",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            BankKey::Ing => "ING",
            BankKey::Cba => "CommBank",
            BankKey::Anz => "ANZ",
            BankKey::Bom => "Bank of Melbourne",
            BankKey::Beyond => "Beyond Bank",
            BankKey::Ibkr => "Interactive Brokers",
        }
    }

    pub fn connector_type(&self) -> &'static str {
        match self {
            BankKey::Ibkr => "broker",
            _ => "bank",
        }
    }
}

impl std::fmt::Display for BankKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for BankKey {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ING" => Ok(BankKey::Ing),
            "CBA" => Ok(BankKey::Cba),
            "ANZ" => Ok(BankKey::Anz),
            "BOM" => Ok(BankKey::Bom),
            "BEYOND" => Ok(BankKey::Beyond),
            "IBKR" => Ok(BankKey::Ibkr),
            _ => Err(format!("Unknown connector key: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankDownloadRun {
    pub id: String,
    pub bank_key: String,
    pub account_name: Option<String>,
    pub status: String,
    pub files_downloaded: i32,
    pub files_skipped: i32,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewBankDownloadRun {
    pub id: String,
    pub bank_key: String,
    pub account_name: Option<String>,
    pub status: String,
    pub started_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankConnectSettings {
    pub download_folder: String,
    pub years_back: u32,
    pub enabled_banks: Vec<String>,
    pub overwrite_existing: bool,
    pub download_timeout_secs: u32,
    pub session_timeout_secs: u32,
    pub retry_attempts: u32,
    pub auto_close_login_window: bool,
    pub log_level: String,
}

impl Default for BankConnectSettings {
    fn default() -> Self {
        Self {
            download_folder: "~/BankStatements".to_string(),
            years_back: 7,
            enabled_banks: vec![
                "ING".to_string(),
                "CBA".to_string(),
                "ANZ".to_string(),
                "BOM".to_string(),
                "BEYOND".to_string(),
                "IBKR".to_string(),
            ],
            overwrite_existing: false,
            download_timeout_secs: 30,
            session_timeout_secs: 120,
            retry_attempts: 3,
            auto_close_login_window: true,
            log_level: "info".to_string(),
        }
    }
}

/// A single transaction row scraped from bank DOM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapedTransaction {
    pub date: String, // "2024-01-15" (YYYY-MM-DD)
    pub description: String,
    pub amount: f64, // negative = debit, positive = credit
    pub balance: Option<f64>,
    pub reference: Option<String>, // bank's own transaction ID if available
    pub transaction_type: Option<String>, // "DEBIT", "CREDIT", "TRANSFER"
}

/// Account metadata scraped from bank DOM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapedAccount {
    pub bank_key: String,
    pub account_name: String,   // "Orange Everyday"
    pub account_number: String, // "1234 5678" (display format, spaces stripped on use)
    pub bsb: Option<String>,    // "923-100"
    pub currency: String,       // "AUD"
    pub account_type: String,   // "TRANSACTION", "SAVINGS", "INVESTMENT"
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

/// Agent step event for the live log strip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStepEvent {
    pub bank_key: String,
    pub step: String, // "navigate", "scrape", "import", "done", "error"
    pub message: String,
    pub detail: Option<String>,
    pub timestamp: String, // ISO 8601
}

/// Generate a stable SHA-256 fingerprint for a scraped transaction.
/// Same transaction always produces the same key → safe for SQLite upsert deduplication.
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

#[cfg(test)]
mod bank_connect_model_tests {
    use super::*;

    fn sample_tx() -> ScrapedTransaction {
        ScrapedTransaction {
            date: "2024-01-15".into(),
            description: "WOOLWORTHS 0123".into(),
            amount: -42.50,
            balance: Some(1234.56),
            reference: None,
            transaction_type: Some("DEBIT".into()),
        }
    }

    #[test]
    fn idempotency_key_is_deterministic() {
        let tx = sample_tx();
        let k1 = make_idempotency_key("ING", "12345678", &tx);
        let k2 = make_idempotency_key("ING", "12345678", &tx);
        assert_eq!(k1, k2);
        assert_eq!(k1.len(), 64); // SHA-256 hex is always 64 chars
    }

    #[test]
    fn idempotency_key_differs_by_bank() {
        let tx = sample_tx();
        let k_ing = make_idempotency_key("ING", "12345678", &tx);
        let k_cba = make_idempotency_key("CBA", "12345678", &tx);
        assert_ne!(k_ing, k_cba);
    }

    #[test]
    fn idempotency_key_differs_by_amount() {
        let mut tx1 = sample_tx();
        let mut tx2 = sample_tx();
        tx1.amount = -42.50;
        tx2.amount = -42.51;
        let k1 = make_idempotency_key("ING", "12345678", &tx1);
        let k2 = make_idempotency_key("ING", "12345678", &tx2);
        assert_ne!(k1, k2);
    }
}
