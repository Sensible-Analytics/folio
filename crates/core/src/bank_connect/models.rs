use serde::{Deserialize, Serialize};

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
