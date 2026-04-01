//! OFX (Open Financial Exchange) parser module.
//!
//! Supports OFX 2.x XML format for Australian banks.

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::errors::{Error, ValidationError};
use crate::Result;

/// OFX transaction model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfxTransaction {
    pub trn_type: String,
    pub date_posted: String,
    pub amount: f64,
    pub fitid: String,
    pub name: String,
    pub memo: Option<String>,
    pub account_id: String,
    pub bank_id: Option<String>,
}

/// Result of parsing an OFX file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfxParseResult {
    pub account_id: String,
    pub bank_id: Option<String>,
    pub account_type: String,
    pub currency: String,
    pub transactions: Vec<OfxTransaction>,
    pub start_date: String,
    pub end_date: String,
    pub errors: Vec<String>,
}

/// Parse OFX content (2.x XML format).
pub fn parse_ofx(content: &[u8]) -> Result<OfxParseResult> {
    let mut errors = Vec::new();

    let content_str = String::from_utf8_lossy(content).into_owned();

    if !content_str.contains("<OFX>") && !content_str.contains("OFXHEADER") {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "Invalid OFX file: missing OFX header".to_string(),
        )));
    }

    // Extract account info
    let (account_id, bank_id, account_type, currency) =
        extract_account_info(&content_str, &mut errors);

    // Extract date range
    let (start_date, end_date) = extract_date_range(&content_str);

    // Extract transactions
    let transactions =
        extract_transactions(&content_str, &account_id, bank_id.as_deref(), &mut errors);

    Ok(OfxParseResult {
        account_id,
        bank_id,
        account_type,
        currency,
        transactions,
        start_date,
        end_date,
        errors,
    })
}

#[allow(clippy::ptr_arg)]
fn extract_account_info(
    content: &str,
    _errors: &mut Vec<String>,
) -> (String, Option<String>, String, String) {
    let account_id = extract_tag(content, "ACCTID").unwrap_or_else(|| "UNKNOWN".to_string());
    let bank_id = extract_tag(content, "BANKID");
    let account_type = extract_tag(content, "ACCTTYPE").unwrap_or_else(|| "CHECKING".to_string());
    let currency = extract_tag(content, "CURDEF").unwrap_or_else(|| "AUD".to_string());

    (account_id, bank_id, account_type, currency)
}

fn extract_date_range(content: &str) -> (String, String) {
    let start_date = extract_tag(content, "DTSTART")
        .map(|s| normalize_ofx_date(&s))
        .unwrap_or_default();
    let end_date = extract_tag(content, "DTEND")
        .map(|s| normalize_ofx_date(&s))
        .unwrap_or_default();
    (start_date, end_date)
}

#[allow(clippy::ptr_arg)]
fn extract_transactions(
    content: &str,
    account_id: &str,
    bank_id: Option<&str>,
    errors: &mut Vec<String>,
) -> Vec<OfxTransaction> {
    let mut transactions = Vec::new();

    // Find BANKTRANLIST block first (handle whitespace in tags)
    let banktranlist_re = Regex::new(r"<BANKTRANLIST\s*>(.*?)</BANKTRANLIST\s*>").ok();
    let tranlist_content = banktranlist_re
        .and_then(|re| re.captures(content))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or(content); // If BANKTRANLIST not found, search whole content

    // Find all STMTTRN blocks (case-insensitive, handle whitespace)
    let stmttrn_re = Regex::new(r"(?i)<STMTTRN\s*>([\s\S]*?)</STMTTRN\s*>").unwrap();
    for caps in stmttrn_re.captures_iter(tranlist_content) {
        let block = caps.get(1).map(|m| m.as_str()).unwrap_or("");

        let trn_type = extract_tag(block, "TRNTYPE").unwrap_or_else(|| "OTHER".to_string());
        let date_posted = extract_tag(block, "DTPOSTED")
            .map(|s| normalize_ofx_date(&s))
            .unwrap_or_else(|| "1900-01-01".to_string());
        let amount_str = extract_tag(block, "TRNAMT").unwrap_or_else(|| "0".to_string());
        let amount = parse_ofx_amount(&amount_str);
        let fitid = extract_tag(block, "FITID").unwrap_or_else(|| {
            let fallback = format!("{}_{}", date_posted, amount);
            generate_fitid_fallback(&fallback)
        });
        let name = extract_tag(block, "NAME").unwrap_or_default();
        let memo = extract_tag(block, "MEMO");

        let tx = OfxTransaction {
            trn_type,
            date_posted,
            amount,
            fitid,
            name,
            memo,
            account_id: account_id.to_string(),
            bank_id: bank_id.map(String::from),
        };
        transactions.push(tx);
    }

    if transactions.is_empty() {
        errors.push("No transactions found in OFX file".to_string());
    }

    transactions
}

fn extract_tag(content: &str, tag: &str) -> Option<String> {
    // Try XML-style tag
    let xml_pattern = format!("<{}>([^<]+)</{}>", tag, tag);
    if let Ok(re) = Regex::new(&xml_pattern) {
        if let Some(caps) = re.captures(content) {
            return Some(
                caps.get(1)
                    .map(|m| m.as_str().trim().to_string())
                    .unwrap_or_default(),
            );
        }
    }

    // Try SGML-style tag (no closing tag)
    let sgml_pattern = format!("<{}>", tag);
    if let Some(start) = content.find(&sgml_pattern) {
        let value_start = start + sgml_pattern.len();
        // Find end (newline or next tag)
        let end = content[value_start..]
            .find('\n')
            .or_else(|| content[value_start..].find("  "))
            .map(|p| value_start + p)
            .unwrap_or(content.len());
        let value = content[value_start..end].trim().to_string();
        if !value.is_empty() {
            return Some(value);
        }
    }

    None
}

fn normalize_ofx_date(date_str: &str) -> String {
    // OFX date format: YYYYMMDD or YYYYMMDDHHMMSS
    let digits: String = date_str.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.len() >= 8 {
        let year = &digits[0..4];
        let month = &digits[4..6];
        let day = &digits[6..8];
        return format!("{}-{}-{}", year, month, day);
    }

    date_str.to_string()
}

fn parse_ofx_amount(s: &str) -> f64 {
    s.trim().replace(',', "").parse().unwrap_or(0.0)
}

fn generate_fitid_fallback(input: &str) -> String {
    format!("{:x}", Sha256::digest(input.as_bytes()))[..32].to_string()
}

/// Generate idempotency key from OFX transaction.
#[allow(dead_code)]
pub fn generate_ofx_idempotency_key(
    account_id: &str,
    fitid: &str,
    date: &str,
    amount: f64,
) -> String {
    let input = format!("OFX|{}|{}|{}|{:.2}", account_id, fitid, date, amount);
    format!("{:x}", Sha256::digest(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_ofx() {
        let ofx = r#"<?xml version="1.0"?>
<OFX>
  <BANKMSGSRSV1>
    <STMTTRNRS>
      <STMTRS>
        <BANKACCTFROM>
          <BANKID>012-000</BANKID>
          <ACCTID>123456789</ACCTID>
          <ACCTTYPE>CHECKING</ACCTTYPE>
        </BANKACCTFROM>
        <BANKTRANLIST>
          <DTSTART>20260301000000</DTSTART>
          <DTEND>20260326000000</DTEND>
          <STMTTRN>
            <TRNTYPE>DEBIT</TRNTYPE>
            <DTPOSTED>20260315</DTPOSTED>
            <TRNAMT>-142.50</TRNAMT>
            <FITID>12345</FITID>
            <NAME>Grocery Store</NAME>
            <MEMO>Weekly shopping</MEMO>
          </STMTTRN>
          <STMTTRN>
            <TRNTYPE>CREDIT</TRNTYPE>
            <DTPOSTED>20260320</DTPOSTED>
            <TRNAMT>5000.00</TRNAMT>
            <FITID>12346</FITID>
            <NAME>EMPLOYER PAY</NAME>
          </STMTTRN>
        </BANKTRANLIST>
        <LEDGERBAL>
          <BALAMT>5432.10</BALAMT>
          <DTASOF>20260326000000</DTASOF>
        </LEDGERBAL>
      </STMTRS>
    </STMTTRNRS>
  </BANKMSGSRSV1>
</OFX>"#;

        let result = parse_ofx(ofx.as_bytes()).unwrap();
        assert_eq!(result.account_id, "123456789");
        assert_eq!(result.bank_id, Some("012-000".to_string()));
        assert_eq!(result.transactions.len(), 2);
        assert_eq!(result.transactions[0].amount, -142.50);
        assert_eq!(result.transactions[1].amount, 5000.00);
    }

    #[test]
    fn test_normalize_ofx_date() {
        assert_eq!(normalize_ofx_date("20260315"), "2026-03-15");
        assert_eq!(normalize_ofx_date("20260315120000"), "2026-03-15");
    }

    #[test]
    fn test_parse_ofx_amount() {
        assert_eq!(parse_ofx_amount("1234.56"), 1234.56);
        assert_eq!(parse_ofx_amount("-1234.56"), -1234.56);
        assert_eq!(parse_ofx_amount("1,234.56"), 1234.56);
    }

    #[test]
    fn test_extract_tag() {
        let content = "<NAME>Test Payee</NAME><MEMO>Test memo</MEMO>";
        assert_eq!(extract_tag(content, "NAME"), Some("Test Payee".to_string()));
        assert_eq!(extract_tag(content, "MEMO"), Some("Test memo".to_string()));
        assert_eq!(extract_tag(content, "MISSING"), None);
    }
}
