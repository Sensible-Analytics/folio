//! Australian bank CSV parser module.
//!
//! Supports auto-detection and parsing of CSV exports from:
//! - Commonwealth Bank (CBA)
//! - Westpac
//! - St. George
//! - ANZ
//! - NAB

use csv::{ReaderBuilder, Terminator};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::errors::{Error, ValidationError};
use crate::Result;

/// Bank transaction model from CSV parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankTransaction {
    /// Normalized date in YYYY-MM-DD format.
    pub date: String,
    /// Transaction description/payee.
    pub description: String,
    /// Amount in dollars (positive = credit, negative = debit).
    pub amount: f64,
    /// Running balance if available.
    pub balance: Option<f64>,
    /// Bank reference number if available.
    pub reference: Option<String>,
    /// Raw transaction type if detected.
    pub transaction_type: Option<String>,
    /// Bank identifier (CBA, WBC, STG, ANZ, NAB).
    pub bank_code: Option<String>,
}

/// Result of parsing an Australian bank CSV.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedBankCsvResult {
    /// Detected bank type.
    pub bank_type: String,
    /// Account number if detected.
    pub account_number: Option<String>,
    /// Account name if detected.
    pub account_name: Option<String>,
    /// Currency code (AUD).
    pub currency: String,
    /// Parsed transactions.
    pub transactions: Vec<BankTransaction>,
    /// Any errors encountered.
    pub errors: Vec<String>,
    /// Total transactions count.
    pub transaction_count: usize,
}

/// Australian bank type detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AustralianBank {
    /// Commonwealth Bank.
    CommBank,
    /// Westpac Banking Corporation.
    Westpac,
    /// St. George Bank (now part of Westpac).
    StGeorge,
    /// Australia and New Zealand Banking Group.
    Anz,
    /// National Australia Bank.
    Nab,
    /// Unable to detect bank.
    Unknown,
}

impl AustralianBank {
    pub fn code(&self) -> &'static str {
        match self {
            AustralianBank::CommBank => "CBA",
            AustralianBank::Westpac => "WBC",
            AustralianBank::StGeorge => "STG",
            AustralianBank::Anz => "ANZ",
            AustralianBank::Nab => "NAB",
            AustralianBank::Unknown => "UNK",
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            AustralianBank::CommBank => "Commonwealth Bank",
            AustralianBank::Westpac => "Westpac",
            AustralianBank::StGeorge => "St. George",
            AustralianBank::Anz => "ANZ",
            AustralianBank::Nab => "NAB",
            AustralianBank::Unknown => "Unknown",
        }
    }
}

/// Detect bank type from CSV headers.
pub fn detect_bank_type(headers: &[String]) -> AustralianBank {
    let headers_lower: Vec<String> = headers.iter().map(|h| h.to_lowercase()).collect();
    let headers_str = headers_lower.join(",");

    // Westpac detection (most specific)
    if headers_str.contains("transaction date")
        && (headers_str.contains("closing balance") || headers_str.contains("transaction amount"))
    {
        return AustralianBank::Westpac;
    }

    // CBA detection - typical headers include Debit, Credit, Balance
    if headers_lower.iter().any(|h| h.contains("debit"))
        && headers_lower.iter().any(|h| h.contains("credit"))
        && headers_lower.iter().any(|h| h.contains("balance"))
    {
        // Check if it's not Westpac extended format
        if !headers_str.contains("closing balance") {
            return AustralianBank::CommBank;
        }
    }

    // St. George / Westpac Business Online
    if headers_str.contains("transaction date")
        && headers_lower.iter().any(|h| h.contains("credit"))
        && headers_lower.iter().any(|h| h.contains("debit"))
    {
        return AustralianBank::StGeorge;
    }

    // ANZ detection
    if (headers_lower.iter().any(|h| h.contains("bsb")) || headers_str.contains("anz"))
        && (headers_str.contains("transaction date") || headers_str.contains("transaction"))
    {
        return AustralianBank::Anz;
    }

    // NAB detection
    if headers_str.contains("nab") && headers_str.contains("transaction") {
        return AustralianBank::Nab;
    }

    // Generic debit/credit format (likely CBA or similar)
    if headers_lower.iter().any(|h| h.contains("debit"))
        && headers_lower.iter().any(|h| h.contains("credit"))
    {
        return AustralianBank::CommBank;
    }

    // Fallback heuristics
    if headers_str.contains("transaction date") && headers_str.contains("closing balance") {
        return AustralianBank::Westpac;
    }

    if headers_str.contains("debit amount") || headers_str.contains("credit amount") {
        return AustralianBank::Anz;
    }

    AustralianBank::Unknown
}

/// Parse Australian bank CSV content.
pub fn parse_australian_bank_csv(content: &[u8]) -> Result<ParsedBankCsvResult> {
    let mut errors = Vec::new();

    // Decode content
    let content_str = decode_content(content, &mut errors);

    // Detect delimiter
    let delimiter = detect_csv_delimiter(&content_str);

    // Parse CSV
    let delimiter_byte = delimiter as u8;
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter_byte)
        .has_headers(true)
        .flexible(true)
        .terminator(Terminator::Any(b'\n'))
        .from_reader(content_str.as_bytes());

    // Get headers
    let headers: Vec<String> = reader
        .headers()
        .map_err(|e| Error::Validation(ValidationError::InvalidInput(e.to_string())))?
        .iter()
        .map(|s| s.to_string())
        .collect();

    if headers.is_empty() {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "CSV file has no headers".to_string(),
        )));
    }

    // Detect bank type
    let bank_type = detect_bank_type(&headers);

    // Build column index mapping
    let col_map = build_column_map(&headers, bank_type);

    // Parse rows
    let mut transactions = Vec::new();
    let mut account_number = None;
    let mut account_name = None;

    for (row_idx, result) in reader.records().enumerate() {
        match result {
            Ok(record) => {
                let row: Vec<&str> = record.iter().collect();

                // Extract account info from first row if available
                if row_idx == 0 {
                    if let Some(idx) = col_map.get("account_number") {
                        if *idx < row.len() {
                            account_number = Some(row[*idx].trim().to_string());
                        }
                    }
                    if let Some(idx) = col_map.get("account_name") {
                        if *idx < row.len() {
                            account_name = Some(row[*idx].trim().to_string());
                        }
                    }
                }

                match parse_transaction_row(&row, &col_map, bank_type) {
                    Ok(Some(tx)) => transactions.push(tx),
                    Ok(None) => {} // Skip empty rows
                    Err(e) => errors.push(format!("Row {}: {}", row_idx + 1, e)),
                }
            }
            Err(e) => {
                errors.push(format!("Failed to parse row {}: {}", row_idx + 1, e));
            }
        }
    }

    let transaction_count = transactions.len();

    Ok(ParsedBankCsvResult {
        bank_type: bank_type.code().to_string(),
        account_number,
        account_name,
        currency: "AUD".to_string(),
        transactions,
        errors,
        transaction_count,
    })
}

/// Decode content handling BOM and encoding issues.
fn decode_content(content: &[u8], errors: &mut Vec<String>) -> String {
    // Check for UTF-8 BOM
    let content_without_bom =
        if content.len() >= 3 && content[0] == 0xEF && content[1] == 0xBB && content[2] == 0xBF {
            &content[3..]
        } else {
            content
        };

    match std::str::from_utf8(content_without_bom) {
        Ok(s) => s.to_string(),
        Err(e) => {
            errors.push(format!("Invalid UTF-8: {}. Using lossy conversion.", e));
            String::from_utf8_lossy(content_without_bom).into_owned()
        }
    }
}

/// Detect CSV delimiter.
fn detect_csv_delimiter(content: &str) -> char {
    let delimiters = [',', ';', '\t'];
    let mut best_delimiter = ',';
    let mut best_score = 0usize;

    for delim in delimiters {
        let score = content
            .lines()
            .take(5)
            .filter(|line| !line.is_empty())
            .filter(|line| line.matches(delim).count() > 2)
            .count();

        if score > best_score {
            best_score = score;
            best_delimiter = delim;
        }
    }

    best_delimiter
}

/// Build column index mapping for bank-specific parsing.
fn build_column_map(headers: &[String], bank: AustralianBank) -> HashMap<String, usize> {
    let mut map = HashMap::new();

    for (idx, header) in headers.iter().enumerate() {
        let h = header.to_lowercase();
        let h_trimmed = h.trim().to_string();

        match bank {
            AustralianBank::Westpac => {
                if h_trimmed.contains("transaction date") {
                    map.insert("date".to_string(), idx);
                } else if h_trimmed.contains("transaction amount") {
                    map.insert("amount".to_string(), idx);
                } else if h_trimmed.contains("closing balance") {
                    map.insert("balance".to_string(), idx);
                } else if h_trimmed.contains("narrative") || h_trimmed.contains("description") {
                    map.insert("description".to_string(), idx);
                } else if h_trimmed.contains("transaction code") {
                    map.insert("code".to_string(), idx);
                } else if h_trimmed.contains("serial") {
                    map.insert("reference".to_string(), idx);
                } else if h_trimmed.contains("account number") {
                    map.insert("account_number".to_string(), idx);
                } else if h_trimmed.contains("account name") {
                    map.insert("account_name".to_string(), idx);
                }
            }
            AustralianBank::CommBank => {
                if h_trimmed.contains("date") && !h_trimmed.contains("effective") {
                    map.insert("date".to_string(), idx);
                } else if h_trimmed.contains("description") || h_trimmed.contains("narrative") {
                    map.insert("description".to_string(), idx);
                } else if h_trimmed.contains("debit") {
                    map.insert("debit".to_string(), idx);
                } else if h_trimmed.contains("credit") {
                    map.insert("credit".to_string(), idx);
                } else if h_trimmed.contains("balance") {
                    map.insert("balance".to_string(), idx);
                }
            }
            AustralianBank::Anz => {
                if h_trimmed.contains("transaction date") || h_trimmed.contains("date") {
                    map.insert("date".to_string(), idx);
                } else if h_trimmed.contains("description") || h_trimmed.contains("narrative") {
                    map.insert("description".to_string(), idx);
                } else if h_trimmed.contains("debit") {
                    map.insert("debit".to_string(), idx);
                } else if h_trimmed.contains("credit") {
                    map.insert("credit".to_string(), idx);
                } else if h_trimmed.contains("balance") {
                    map.insert("balance".to_string(), idx);
                } else if h_trimmed.contains("bsb") {
                    map.insert("bsb".to_string(), idx);
                }
            }
            _ => {
                // Generic column detection
                if h_trimmed.contains("date") {
                    map.entry("date".to_string()).or_insert(idx);
                } else if h_trimmed.contains("description")
                    || h_trimmed.contains("narrative")
                    || h_trimmed.contains("particular")
                {
                    map.entry("description".to_string()).or_insert(idx);
                } else if h_trimmed.contains("amount")
                    || h_trimmed.contains("debit")
                    || h_trimmed.contains("credit")
                {
                    if h_trimmed.contains("debit") {
                        map.insert("debit".to_string(), idx);
                    } else if h_trimmed.contains("credit") {
                        map.insert("credit".to_string(), idx);
                    } else {
                        map.insert("amount".to_string(), idx);
                    }
                } else if h_trimmed.contains("balance") {
                    map.insert("balance".to_string(), idx);
                }
            }
        }
    }

    map
}

/// Parse a single transaction row.
fn parse_transaction_row(
    row: &[&str],
    col_map: &HashMap<String, usize>,
    bank: AustralianBank,
) -> Result<Option<BankTransaction>> {
    // Get date
    let date_str = col_map
        .get("date")
        .and_then(|&idx| row.get(idx))
        .map(|s| s.trim())
        .unwrap_or_default();

    if date_str.is_empty() {
        return Ok(None);
    }

    let date = parse_date(date_str)?;

    // Get description
    let description = col_map
        .get("description")
        .and_then(|&idx| row.get(idx))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Get amount
    let amount = parse_amount(row, col_map, bank)?;

    // Get balance (optional)
    let balance = col_map
        .get("balance")
        .and_then(|&idx| row.get(idx))
        .and_then(|s| parse_balance(s.trim()));

    // Get reference (optional)
    let reference = col_map
        .get("reference")
        .and_then(|&idx| row.get(idx))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(Some(BankTransaction {
        date,
        description,
        amount,
        balance,
        reference,
        transaction_type: None,
        bank_code: Some(bank.code().to_string()),
    }))
}

/// Parse date from various formats.
fn parse_date(s: &str) -> Result<String> {
    let s = s.trim();

    // Try YYYYMMDD (Westpac)
    if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        let year = &s[0..4];
        let month = &s[4..6];
        let day = &s[6..8];
        return Ok(format!("{}-{}-{}", year, month, day));
    }

    // Try DD/MM/YYYY
    let re_dmy = Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{4})$").unwrap();
    if let Some(caps) = re_dmy.captures(s) {
        let day = caps.get(1).map(|m| m.as_str()).unwrap_or("01");
        let month = caps.get(2).map(|m| m.as_str()).unwrap_or("01");
        let year = caps.get(3).map(|m| m.as_str()).unwrap_or("2026");
        return Ok(format!(
            "{}-{:0>2}-{:0>2}",
            year,
            month.parse::<usize>().unwrap_or(1),
            day.parse::<usize>().unwrap_or(1)
        ));
    }

    // Try DD-MM-YYYY or DD-Mmm-YYYY (ANZ format)
    let re_dmy_dash = Regex::new(r"^(\d{1,2})-(\w{3})-(\d{4})$").unwrap();
    if let Some(caps) = re_dmy_dash.captures(s) {
        let day = caps.get(1).map(|m| m.as_str()).unwrap_or("01");
        let month_str = caps.get(2).map(|m| m.as_str()).unwrap_or("Jan");
        let year = caps.get(3).map(|m| m.as_str()).unwrap_or("2026");
        let month = month_name_to_num(month_str);
        return Ok(format!(
            "{}-{:0>2}-{:0>2}",
            year,
            month,
            day.parse::<usize>().unwrap_or(1)
        ));
    }

    // Try YYYY-MM-DD (already ISO)
    let re_ymd = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").unwrap();
    if re_ymd.is_match(s) {
        return Ok(s.to_string());
    }

    Err(Error::Validation(ValidationError::InvalidInput(format!(
        "Unable to parse date: {}",
        s
    ))))
}

/// Convert month name to number.
fn month_name_to_num(month: &str) -> usize {
    match month.to_lowercase().as_str() {
        "jan" => 1,
        "feb" => 2,
        "mar" => 3,
        "apr" => 4,
        "may" => 5,
        "jun" => 6,
        "jul" => 7,
        "aug" => 8,
        "sep" => 9,
        "oct" => 10,
        "nov" => 11,
        "dec" => 12,
        _ => 1,
    }
}

/// Parse amount from row based on bank format.
fn parse_amount(
    row: &[&str],
    col_map: &HashMap<String, usize>,
    bank: AustralianBank,
) -> Result<f64> {
    match bank {
        AustralianBank::CommBank | AustralianBank::Anz => {
            // CBA/ANZ format: separate debit and credit columns
            let debit = col_map
                .get("debit")
                .and_then(|&idx| row.get(idx))
                .and_then(|s| parse_amount_value(s.trim()));

            let credit = col_map
                .get("credit")
                .and_then(|&idx| row.get(idx))
                .and_then(|s| parse_amount_value(s.trim()));

            // Only treat as debit if non-zero
            if let Some(d) = debit {
                if d != 0.0 {
                    return Ok(-d.abs());
                }
            }
            // Use credit if available
            if let Some(c) = credit {
                return Ok(c.abs());
            }
            Ok(0.0)
        }
        _ => {
            // Westpac and generic: single amount column
            if let Some(amount_str) = col_map.get("amount").and_then(|&idx| row.get(idx)) {
                parse_amount_value(amount_str.trim()).ok_or_else(|| {
                    Error::Validation(ValidationError::InvalidInput(format!(
                        "Invalid amount: {}",
                        amount_str
                    )))
                })
            } else {
                Err(Error::Validation(ValidationError::InvalidInput(
                    "No amount column found".to_string(),
                )))
            }
        }
    }
}

/// Parse amount value from string.
fn parse_amount_value(s: &str) -> Option<f64> {
    if s.is_empty() {
        return None;
    }

    let s = s.trim();

    // Remove currency symbols and whitespace
    let s = s.replace(['$', '€', '£', ','], "").replace(" ", "");

    // Handle parentheses as negative (accounting format)
    let (s, negative) = if s.starts_with('(') && s.ends_with(')') {
        (&s[1..s.len() - 1], true)
    } else if s.ends_with('-') {
        (&s[..s.len() - 1], true)
    } else {
        (s.as_str(), false)
    };

    s.parse::<f64>()
        .ok()
        .map(|v| if negative { -v.abs() } else { v })
}

/// Parse balance from string.
fn parse_balance(s: &str) -> Option<f64> {
    if s.is_empty() || s == "-" || s.to_lowercase() == "n/a" {
        return None;
    }
    parse_amount_value(s)
}

/// Generate idempotency key for a bank transaction.
#[allow(dead_code)]
pub fn generate_bank_idempotency_key(
    source_system: &str,
    account_id: &str,
    date: &str,
    amount: f64,
    description: &str,
) -> String {
    let input = format!(
        "BANK|{}|{}|{}|{:.2}|{}",
        source_system,
        account_id,
        date,
        amount,
        description.trim()
    );
    format!("{:x}", Sha256::digest(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_cba_csv() {
        let headers = vec![
            "Date".to_string(),
            "Description".to_string(),
            "Debit".to_string(),
            "Credit".to_string(),
            "Balance".to_string(),
        ];
        assert_eq!(detect_bank_type(&headers), AustralianBank::CommBank);
    }

    #[test]
    fn test_detect_westpac_csv() {
        let headers = vec![
            "Transaction date".to_string(),
            "Account number".to_string(),
            "Account name".to_string(),
            "Currency code".to_string(),
            "Closing balance".to_string(),
            "Transaction amount".to_string(),
            "Transaction code".to_string(),
            "Narrative".to_string(),
        ];
        assert_eq!(detect_bank_type(&headers), AustralianBank::Westpac);
    }

    #[test]
    fn test_parse_date_yyyymmdd() {
        assert_eq!(parse_date("20260325").unwrap(), "2026-03-25");
    }

    #[test]
    fn test_parse_date_ddmmyyyy() {
        assert_eq!(parse_date("25/03/2026").unwrap(), "2026-03-25");
    }

    #[test]
    fn test_parse_date_ddmmmyyyy() {
        assert_eq!(parse_date("25-Mar-2026").unwrap(), "2026-03-25");
    }

    #[test]
    fn test_parse_amount_positive() {
        assert_eq!(parse_amount_value("1,234.56").unwrap(), 1234.56);
        assert_eq!(parse_amount_value("$1,234.56").unwrap(), 1234.56);
    }

    #[test]
    fn test_parse_amount_negative() {
        assert_eq!(parse_amount_value("-1,234.56").unwrap(), -1234.56);
        assert_eq!(parse_amount_value("(1,234.56)").unwrap(), -1234.56);
    }

    #[test]
    fn test_idempotency_key() {
        let key1 = generate_bank_idempotency_key("CBA", "123456", "2026-03-25", 1234.56, "Test");
        let key2 = generate_bank_idempotency_key("CBA", "123456", "2026-03-25", 1234.56, "Test");
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn test_parse_cba_csv() {
        let csv = b"Date,Description,Debit,Credit,Balance\n25/03/2026,Salary,0.00,5000.00,10000.00\n26/03/2026,Woolworths,150.50,0.00,9849.50";

        let result = parse_australian_bank_csv(csv).unwrap();
        assert_eq!(result.bank_type, "CBA");
        assert_eq!(result.transactions.len(), 2);
        assert_eq!(result.transactions[0].amount, 5000.00);
        assert_eq!(result.transactions[1].amount, -150.50);
    }

    #[test]
    fn test_parse_westpac_csv() {
        let csv = b"Transaction date,Account number,Account name,Currency code,Closing balance,Transaction amount,Transaction code,Narrative\n20260325,032000123456,Savings,AUD,5432.10,+150.00,050,Grocery Store\n20260326,032000123456,Savings,AUD,5282.10,-150.00,050,Grocery Store";

        let result = parse_australian_bank_csv(csv).unwrap();
        assert_eq!(result.bank_type, "WBC");
        assert_eq!(result.transactions.len(), 2);
    }
}
