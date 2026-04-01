//! QIF (Quicken Interchange Format) parser module.
//!
//! Supports QIF format for Australian bank statements.

use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::errors::{Error, ValidationError};
use crate::Result;

/// QIF transaction model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QifTransaction {
    pub date: String,
    pub amount: f64,
    pub payee: Option<String>,
    pub memo: Option<String>,
    pub reference: Option<String>,
    pub category: Option<String>,
    pub cleared: bool,
}

/// Result of parsing a QIF file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QifParseResult {
    pub account_type: String,
    pub transactions: Vec<QifTransaction>,
    pub errors: Vec<String>,
}

/// Parse QIF content.
pub fn parse_qif(content: &[u8]) -> Result<QifParseResult> {
    let mut errors = Vec::new();

    let content_str = String::from_utf8_lossy(content).into_owned();
    let lines: Vec<&str> = content_str.lines().collect();

    if lines.is_empty() {
        return Err(Error::Validation(ValidationError::InvalidInput(
            "QIF file is empty".to_string(),
        )));
    }

    // Detect account type from header
    let account_type = detect_account_type(&lines);

    // Parse transactions
    let transactions = parse_qif_transactions(&lines, &mut errors);

    Ok(QifParseResult {
        account_type,
        transactions,
        errors,
    })
}

fn detect_account_type(lines: &[&str]) -> String {
    for line in lines {
        if line.starts_with("!Type:") {
            let t = line.trim_start_matches("!Type:").trim().to_lowercase();
            return match t.as_str() {
                "bank" => "Bank".to_string(),
                "cash" => "Cash".to_string(),
                "invst" => "Investment".to_string(),
                "ccard" => "Credit Card".to_string(),
                _ => t,
            };
        }
    }
    "Bank".to_string()
}

fn parse_qif_transactions(lines: &[&str], errors: &mut Vec<String>) -> Vec<QifTransaction> {
    let mut transactions = Vec::new();
    let mut current_tx: Option<QifTransactionBuilder> = None;

    for line in lines {
        let line = line.trim();

        // Skip empty lines and headers
        if line.is_empty() || line.starts_with('!') {
            continue;
        }

        let code = line.chars().next().unwrap_or(' ');
        let value = &line[1..].trim();

        match code {
            'D' => {
                // Date - save previous transaction if exists
                if let Some(builder) = current_tx.take() {
                    if let Some(tx) = builder.build() {
                        transactions.push(tx);
                    }
                }
                current_tx = Some(QifTransactionBuilder {
                    date: Some(parse_qif_date(value).unwrap_or_else(|| "1900-01-01".to_string())),
                    amount: None,
                    payee: None,
                    memo: None,
                    reference: None,
                    category: None,
                    cleared: false,
                });
            }
            'T' | 'U' => {
                // Amount
                if let Some(ref mut builder) = current_tx {
                    builder.amount = parse_qif_amount(value);
                }
            }
            'N' => {
                // Reference/check number
                if let Some(ref mut builder) = current_tx {
                    builder.reference = Some(value.to_string());
                }
            }
            'P' => {
                // Payee
                if let Some(ref mut builder) = current_tx {
                    builder.payee = Some(value.to_string());
                }
            }
            'M' => {
                // Memo
                if let Some(ref mut builder) = current_tx {
                    builder.memo = Some(value.to_string());
                }
            }
            'L' => {
                // Category
                if let Some(ref mut builder) = current_tx {
                    builder.category = Some(value.to_string());
                }
            }
            'C' => {
                // Cleared status
                if let Some(ref mut builder) = current_tx {
                    builder.cleared = value.to_lowercase() == "*" || value.to_lowercase() == "x";
                }
            }
            '^' => {
                // End of transaction
                if let Some(builder) = current_tx.take() {
                    if let Some(tx) = builder.build() {
                        transactions.push(tx);
                    }
                }
            }
            _ => {}
        }
    }

    // Don't forget the last transaction
    if let Some(builder) = current_tx {
        if let Some(tx) = builder.build() {
            transactions.push(tx);
        }
    }

    if transactions.is_empty() {
        errors.push("No transactions found in QIF file".to_string());
    }

    transactions
}

fn parse_qif_date(s: &str) -> Option<String> {
    let s = s.trim();

    // Australian format: D/M/YY or D/M/YYYY
    let re_dmy = Regex::new(r"^(\d{1,2})/(\d{1,2})/(\d{2,4})$").ok()?;
    if let Some(caps) = re_dmy.captures(s) {
        let day = caps.get(1)?.as_str();
        let month = caps.get(2)?.as_str();
        let year = caps.get(3)?.as_str();

        let year_full = if year.len() == 2 {
            format!("20{}", year)
        } else {
            year.to_string()
        };

        return Some(format!(
            "{}-{:0>2}-{:0>2}",
            year_full,
            month.parse::<usize>().ok()?,
            day.parse::<usize>().ok()?
        ));
    }

    // British format: D-MMM-YY or D-MMM-YYYY
    let re_dmy_dash = Regex::new(r"^(\d{1,2})-(\w{3})-(\d{2,4})$").ok()?;
    if let Some(caps) = re_dmy_dash.captures(s) {
        let day = caps.get(1)?.as_str();
        let month_str = caps.get(2)?.as_str();
        let year = caps.get(3)?.as_str();

        let month = month_name_to_num(month_str);
        let year_full = if year.len() == 2 {
            format!("20{}", year)
        } else {
            year.to_string()
        };

        return Some(format!(
            "{}-{:0>2}-{:0>2}",
            year_full,
            month,
            day.parse::<usize>().ok()?
        ));
    }

    // ISO format: YYYY-MM-DD
    let re_ymd = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").ok()?;
    if let Some(caps) = re_ymd.captures(s) {
        let year = caps.get(1)?.as_str();
        let month = caps.get(2)?.as_str();
        let day = caps.get(3)?.as_str();
        return Some(format!("{}-{}-{}", year, month, day));
    }

    None
}

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

fn parse_qif_amount(s: &str) -> Option<f64> {
    let s = s.trim().replace(',', "");

    if s.is_empty() {
        return None;
    }

    // Handle parentheses as negative
    let (s, negative) = if s.starts_with('(') && s.ends_with(')') {
        (&s[1..s.len() - 1], true)
    } else {
        (s.as_str(), s.starts_with('-'))
    };

    s.parse::<f64>()
        .ok()
        .map(|v| if negative { -v.abs() } else { v })
}

struct QifTransactionBuilder {
    date: Option<String>,
    amount: Option<f64>,
    payee: Option<String>,
    memo: Option<String>,
    reference: Option<String>,
    category: Option<String>,
    cleared: bool,
}

impl QifTransactionBuilder {
    fn build(self) -> Option<QifTransaction> {
        Some(QifTransaction {
            date: self.date?,
            amount: self.amount.unwrap_or(0.0),
            payee: self.payee,
            memo: self.memo,
            reference: self.reference,
            category: self.category,
            cleared: self.cleared,
        })
    }
}

/// Generate idempotency key from QIF transaction.
#[allow(dead_code)]
pub fn generate_qif_idempotency_key(
    account_type: &str,
    date: &str,
    amount: f64,
    payee: Option<&str>,
) -> String {
    let input = format!(
        "QIF|{}|{}|{:.2}|{}",
        account_type,
        date,
        amount,
        payee.unwrap_or("")
    );
    format!("{:x}", Sha256::digest(input.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_qif() {
        let qif = r#"!Type:Bank
D25/03/2026
T-142.50
N1005
PGrocery Store
MGrocery run
LFood
^
D26/03/2026
T5000.00
PEMPLOYER PAY
LSalary
^
D27/03/2026
T-50.00
NATM Withdrawal
^
"#;

        let result = parse_qif(qif.as_bytes()).unwrap();
        assert_eq!(result.account_type, "Bank");
        assert_eq!(result.transactions.len(), 3);
        assert_eq!(result.transactions[0].amount, -142.50);
        assert_eq!(
            result.transactions[0].payee,
            Some("Grocery Store".to_string())
        );
        assert_eq!(result.transactions[1].amount, 5000.00);
        assert_eq!(result.transactions[2].amount, -50.00);
    }

    #[test]
    fn test_parse_qif_date() {
        assert_eq!(parse_qif_date("25/03/2026").unwrap(), "2026-03-25");
        assert_eq!(parse_qif_date("25/03/26").unwrap(), "2026-03-25");
        assert_eq!(parse_qif_date("25-Mar-2026").unwrap(), "2026-03-25");
        assert_eq!(parse_qif_date("2026-03-25").unwrap(), "2026-03-25");
    }

    #[test]
    fn test_parse_qif_amount() {
        assert_eq!(parse_qif_amount("1234.56").unwrap(), 1234.56);
        assert_eq!(parse_qif_amount("-1234.56").unwrap(), -1234.56);
        assert_eq!(parse_qif_amount("(1234.56)").unwrap(), -1234.56);
        assert_eq!(parse_qif_amount("1,234.56").unwrap(), 1234.56);
    }

    #[test]
    fn test_detect_account_type() {
        let bank = ["!Type:Bank"];
        let cash = ["!Type:Cash"];
        let invst = ["!Type:Invst"];
        let ccard = ["!Type:Ccard"];

        assert_eq!(detect_account_type(&bank), "Bank");
        assert_eq!(detect_account_type(&cash), "Cash");
        assert_eq!(detect_account_type(&invst), "Investment");
        assert_eq!(detect_account_type(&ccard), "Credit Card");
    }
}
