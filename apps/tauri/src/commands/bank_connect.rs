use log::{debug, info};
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;

use crate::context::ServiceContext;
use wealthfolio_core::accounts::{NewAccount, TrackingMode};
use wealthfolio_core::bank_connect::{
    BankConnectSettings, BankDownloadRun, BankKey, NewBankDownloadRun, ScrapedAccount,
};

// ─── Event Payloads ─────────────────────────────────────────────────────────

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BankLoginDetectedPayload {
    bank_key: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BankProgressPayload {
    pub bank_key: String,
    pub level: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BankWindowClosedPayload {
    bank_key: String,
}

// ─── Settings persistence (JSON file in app data dir) ───────────────────────

fn settings_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(app_data.join("bank_connect_settings.json"))
}

fn load_settings(app: &AppHandle) -> BankConnectSettings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(_) => return BankConnectSettings::default(),
    };
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings_to_file(app: &AppHandle, settings: &BankConnectSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Map a bank-scraped account type string to the canonical Wealthfolio account type.
fn map_account_type(raw: &str) -> String {
    match raw.to_uppercase().as_str() {
        "SAVINGS" => "SAVINGS".to_string(),
        "INVESTMENT" => "TRADING".to_string(),
        _ => "CHECKING".to_string(),
    }
}

/// Find an existing account by provider_account_id, or create a new one.
/// provider_account_id format: "{BANK_KEY}:{account_number_stripped}"
async fn find_or_create_bank_account(
    ctx: &ServiceContext,
    scraped: &ScrapedAccount,
) -> Result<String, String> {
    let account_number = scraped.account_number.replace(' ', "");
    let provider_id = format!("{}:{}", scraped.bank_key, account_number);

    // Check if account already exists
    let accounts = ctx
        .account_service()
        .get_all_accounts()
        .map_err(|e| e.to_string())?;

    if let Some(existing) = accounts
        .iter()
        .find(|a| a.provider_account_id.as_deref() == Some(&provider_id))
    {
        return Ok(existing.id.clone());
    }

    // Create new account
    let new_account = NewAccount {
        id: Some(Uuid::new_v4().to_string()),
        name: format!("{} - {}", scraped.bank_key, scraped.account_name),
        account_type: map_account_type(&scraped.account_type),
        group: None,
        currency: scraped.currency.clone(),
        is_default: false,
        is_active: true,
        platform_id: None,
        account_number: Some(account_number),
        meta: None,
        provider: Some("BANK_CONNECT".to_string()),
        provider_account_id: Some(provider_id),
        is_archived: false,
        tracking_mode: TrackingMode::Transactions,
    };

    ctx.account_service()
        .create_account(new_account)
        .await
        .map(|a| a.id)
        .map_err(|e| e.to_string())
}

// ─── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_bank_connect_settings(app: AppHandle) -> Result<BankConnectSettings, String> {
    Ok(load_settings(&app))
}

#[tauri::command]
pub async fn save_bank_connect_settings(
    app: AppHandle,
    settings: BankConnectSettings,
) -> Result<(), String> {
    save_settings_to_file(&app, &settings)
}

#[tauri::command]
pub async fn list_bank_download_runs(
    bank_key: Option<String>,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<Vec<BankDownloadRun>, String> {
    debug!("Listing bank download runs, bank_key={:?}", bank_key);
    state
        .bank_connect_repository()
        .list_runs(bank_key.as_deref())
        .map_err(|e| format!("Failed to list bank download runs: {}", e))
}

#[tauri::command]
pub async fn open_bank_window(bank_key: String, app: AppHandle) -> Result<(), String> {
    let parsed_key: BankKey = bank_key.parse().map_err(|e: String| e)?;
    let label = format!("bank-{}", bank_key.to_lowercase());
    let login_url = parsed_key.login_url();
    let post_login = parsed_key.post_login_pattern().to_string();
    let bank_key_clone = bank_key.clone();
    let app_for_nav = app.clone();

    debug!("Opening bank window for {} at {}", bank_key, login_url);

    // Close existing window if open
    if let Some(existing) = app.get_webview_window(&label) {
        let _ = existing.close();
    }

    // Get app data dir for session isolation
    let session_dir = app
        .path()
        .app_data_dir()
        .map(|p| p.join("bank-sessions").join(&bank_key))
        .ok();

    let url = WebviewUrl::External(tauri::Url::parse(login_url).map_err(|e| e.to_string())?);

    // on_navigation is a builder method in Tauri v2
    let mut builder = WebviewWindowBuilder::new(&app, &label, url)
        .title(format!("{} - Bank Connect", parsed_key.display_name()))
        .inner_size(1200.0, 800.0)
        .resizable(true)
        .on_navigation(move |nav_url| {
            let url_str = nav_url.as_str();
            if url_str.contains(&post_login) {
                info!("Bank login detected for {}: {}", bank_key_clone, url_str);
                let _ = app_for_nav.emit(
                    "bank://login-detected",
                    BankLoginDetectedPayload {
                        bank_key: bank_key_clone.clone(),
                    },
                );
            }
            true
        });

    if let Some(dir) = session_dir {
        builder = builder.data_directory(dir);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    // Emit window closed event on close
    let app_for_close = app.clone();
    let bank_key_for_close = bank_key.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Destroyed) {
            let _ = app_for_close.emit(
                "bank://window-closed",
                BankWindowClosedPayload {
                    bank_key: bank_key_for_close.clone(),
                },
            );
        }
    });

    info!("Bank window opened for {}", bank_key);
    Ok(())
}

#[tauri::command]
pub async fn close_bank_window(bank_key: String, app: AppHandle) -> Result<(), String> {
    let label = format!("bank-{}", bank_key.to_lowercase());
    if let Some(window) = app.get_webview_window(&label) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn start_bank_download(
    bank_key: String,
    app: AppHandle,
    state: State<'_, Arc<ServiceContext>>,
) -> Result<String, String> {
    let settings = load_settings(&app);
    let run_id = Uuid::new_v4().to_string();
    let started_at = chrono::Utc::now().to_rfc3339();

    // Create run record
    let new_run = NewBankDownloadRun {
        id: run_id.clone(),
        bank_key: bank_key.clone(),
        account_name: None,
        status: "running".to_string(),
        started_at: started_at.clone(),
    };
    state
        .bank_connect_repository()
        .create_run(new_run)
        .map_err(|e| format!("Failed to create download run: {}", e))?;

    // Get the automation script
    let script = crate::banks::get_automation_script(&bank_key, settings.years_back)
        .ok_or_else(|| format!("No automation script for bank: {}", bank_key))?;

    // Inject script into bank window
    let label = format!("bank-{}", bank_key.to_lowercase());
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Bank window not open for {}", bank_key))?;

    // Emit start event
    let _ = app.emit(
        "bank://progress",
        BankProgressPayload {
            bank_key: bank_key.clone(),
            level: "info".to_string(),
            message: "Starting download automation...".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    );

    window
        .eval(&script)
        .map_err(|e| format!("Failed to inject automation script: {}", e))?;

    info!("Bank download started for run {}", run_id);
    Ok(run_id)
}

#[cfg(test)]
mod bank_connect_tests {
    use wealthfolio_core::bank_connect::models::ScrapedAccount;

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
    fn provider_id_format() {
        let scraped = sample_scraped_account();
        let account_number = scraped.account_number.replace(' ', "");
        let provider_id = format!("{}:{}", scraped.bank_key, account_number);
        assert_eq!(provider_id, "ING:12345678");
    }

    #[test]
    fn provider_id_strips_spaces() {
        let mut scraped = sample_scraped_account();
        scraped.account_number = "1234 5678".into();
        let account_number = scraped.account_number.replace(' ', "");
        let provider_id = format!("{}:{}", scraped.bank_key, account_number);
        assert_eq!(provider_id, "ING:12345678");
    }

    #[test]
    fn map_account_type_defaults_to_checking() {
        assert_eq!(super::map_account_type("TRANSACTION"), "CHECKING");
        assert_eq!(super::map_account_type("SAVINGS"), "SAVINGS");
        assert_eq!(super::map_account_type("INVESTMENT"), "TRADING");
        assert_eq!(super::map_account_type("unknown"), "CHECKING");
    }
}
