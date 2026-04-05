//! Bank transaction to Activity mapper.
//!
//! Converts bank transactions from CSV/OFX/QIF parsers to the canonical NewActivity model.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::activities::{ActivityStatus, NewActivity};

pub struct BankMapperConfig {
    pub account_id: String,
    pub currency: String,
    pub source_system: String,
}

impl BankMapperConfig {
    pub fn new(account_id: String, currency: String, source: &str) -> Self {
        Self {
            account_id,
            currency,
            source_system: source.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankTransactionInput {
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub reference: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MappedActivity {
    pub activity: NewActivity,
    pub classification_reason: String,
}

pub fn map_bank_transactions(
    transactions: Vec<BankTransactionInput>,
    config: &BankMapperConfig,
) -> Vec<MappedActivity> {
    transactions
        .into_iter()
        .map(|tx| map_single_transaction(&tx, config))
        .collect()
}

fn map_single_transaction(tx: &BankTransactionInput, config: &BankMapperConfig) -> MappedActivity {
    let (activity_type, subtype, reason) = classify_transaction(tx);

    let amount_decimal = Decimal::from_f64_retain(tx.amount).unwrap_or(Decimal::ZERO);

    let idempotency_key = generate_bank_activity_key(
        &config.source_system,
        &config.account_id,
        &tx.date,
        tx.amount,
        &tx.description,
    );

    let activity = NewActivity {
        id: None,
        account_id: config.account_id.clone(),
        symbol: None,
        activity_type: activity_type.to_string(),
        subtype,
        activity_date: tx.date.clone(),
        quantity: None,
        unit_price: None,
        currency: config.currency.clone(),
        fee: None,
        amount: Some(amount_decimal),
        status: Some(ActivityStatus::Posted),
        notes: Some(tx.description.clone()),
        fx_rate: None,
        metadata: None,
        needs_review: None,
        source_system: Some(config.source_system.clone()),
        source_record_id: tx.reference.clone(),
        source_group_id: None,
        idempotency_key: Some(idempotency_key),
    };

    MappedActivity {
        activity,
        classification_reason: reason,
    }
}

#[allow(clippy::needless_return)]
fn classify_transaction(tx: &BankTransactionInput) -> (String, Option<String>, String) {
    let desc_upper = tx.description.to_uppercase();

    // Check QIF category first if available
    if let Some(ref cat) = tx.category {
        let cat_upper = cat.to_uppercase();

        if cat_upper.contains("SALARY")
            || cat_upper.contains("WAGES")
            || cat_upper.contains("PAYROLL")
        {
            return (
                "DEPOSIT".to_string(),
                Some("SALARY".to_string()),
                "QIF category: Salary".to_string(),
            );
        }
        if cat_upper.contains("INTEREST") {
            return (
                "INTEREST".to_string(),
                None,
                "QIF category: Interest".to_string(),
            );
        }
        if cat_upper.contains("DIVIDEND") || cat_upper.contains("DIV") {
            return (
                "DIVIDEND".to_string(),
                None,
                "QIF category: Dividend".to_string(),
            );
        }
        if cat_upper.contains("TRANSFER") && tx.amount > 0.0 {
            return (
                "TRANSFER_IN".to_string(),
                None,
                "QIF category: Transfer In".to_string(),
            );
        }
        if cat_upper.contains("TRANSFER") && tx.amount < 0.0 {
            return (
                "TRANSFER_OUT".to_string(),
                None,
                "QIF category: Transfer Out".to_string(),
            );
        }
        if cat_upper.contains("FEE") || cat_upper.contains("CHARGE") {
            return ("FEE".to_string(), None, "QIF category: Fee".to_string());
        }
        if cat_upper.contains("FOOD") || cat_upper.contains("GROCERY") || cat_upper.contains("SHOP")
        {
            return (
                "WITHDRAWAL".to_string(),
                Some("PURCHASE".to_string()),
                "QIF category: Shopping".to_string(),
            );
        }
        if cat_upper.contains("UTIL") || cat_upper.contains("BILL") {
            return (
                "WITHDRAWAL".to_string(),
                Some("PAYMENT".to_string()),
                "QIF category: Bill Payment".to_string(),
            );
        }
    }

    // Amount-based primary classification
    if tx.amount > 0.0 {
        // Credits
        if contains_any(
            &desc_upper,
            &["PAYROLL", "SALARY", "WAGES", "EMPLOYER", "PAYG"],
        ) {
            return (
                "DEPOSIT".to_string(),
                Some("SALARY".to_string()),
                "Description contains: Payroll/Salary".to_string(),
            );
        }
        if contains_any(&desc_upper, &["INTEREST"]) {
            return (
                "INTEREST".to_string(),
                None,
                "Description contains: Interest".to_string(),
            );
        }
        if contains_any(&desc_upper, &["DIVIDEND", "DIV"]) {
            return (
                "DIVIDEND".to_string(),
                None,
                "Description contains: Dividend".to_string(),
            );
        }
        if contains_any(&desc_upper, &["REFUND", "REBATE", "CASHBACK"]) {
            return (
                "CREDIT".to_string(),
                Some("REFUND".to_string()),
                "Description contains: Refund".to_string(),
            );
        }
        if contains_any(&desc_upper, &["TRANSFER IN", "FROM ", "TRANSFER FROM"]) {
            return (
                "TRANSFER_IN".to_string(),
                None,
                "Description contains: Transfer In".to_string(),
            );
        }
        return (
            "DEPOSIT".to_string(),
            None,
            format!("Amount positive: ${:.2}", tx.amount),
        );
    } else {
        // Debits
        if contains_any(&desc_upper, &["BPAY", "BILL", "PAYMENT"]) {
            return (
                "WITHDRAWAL".to_string(),
                Some("PAYMENT".to_string()),
                "Description contains: Payment".to_string(),
            );
        }
        if contains_any(&desc_upper, &["ATM", "CASH WITHDRAWAL", "CASH OUT"]) {
            return (
                "WITHDRAWAL".to_string(),
                Some("ATM".to_string()),
                "Description contains: ATM".to_string(),
            );
        }
        if contains_any(&desc_upper, &["TRANSFER OUT", "TO ", "TRANSFER TO"])
            && !contains_any(&desc_upper, &["FROM"])
        {
            return (
                "TRANSFER_OUT".to_string(),
                None,
                "Description contains: Transfer Out".to_string(),
            );
        }
        if contains_any(
            &desc_upper,
            &["FEE", "CHARGE", "MAINTENANCE", "SERVICE CHARGE"],
        ) {
            return (
                "FEE".to_string(),
                None,
                "Description contains: Fee".to_string(),
            );
        }
        if contains_any(
            &desc_upper,
            &[
                "GROCERY",
                "WOOLWORTHS",
                "COLES",
                "SHOPPING",
                "STORE",
                "PURCHASE",
            ],
        ) {
            return (
                "WITHDRAWAL".to_string(),
                Some("PURCHASE".to_string()),
                "Description contains: Shopping".to_string(),
            );
        }
        if contains_any(
            &desc_upper,
            &["PETROL", "FUEL", "SHELL", "BP ", "CALTEX", "MOBIL"],
        ) {
            return (
                "WITHDRAWAL".to_string(),
                Some("FUEL".to_string()),
                "Description contains: Fuel".to_string(),
            );
        }
        if contains_any(
            &desc_upper,
            &[
                "RESTAURANT",
                "CAFE",
                "COFFEE",
                "UBER EATS",
                "MENU LOG",
                "DOORDASH",
            ],
        ) {
            return (
                "WITHDRAWAL".to_string(),
                Some("DINING".to_string()),
                "Description contains: Dining".to_string(),
            );
        }
        if contains_any(
            &desc_upper,
            &[
                "SUBSCRIPTION",
                "NETFLIX",
                "SPOTIFY",
                "STREAMING",
                "MEMBERSHIP",
            ],
        ) {
            return (
                "WITHDRAWAL".to_string(),
                Some("SUBSCRIPTION".to_string()),
                "Description contains: Subscription".to_string(),
            );
        }
        if contains_any(&desc_upper, &["INSURANCE", "HEALTH"]) {
            return (
                "WITHDRAWAL".to_string(),
                Some("INSURANCE".to_string()),
                "Description contains: Insurance".to_string(),
            );
        }
        if contains_any(&desc_upper, &["RENT"]) {
            return (
                "WITHDRAWAL".to_string(),
                Some("RENT".to_string()),
                "Description contains: Rent".to_string(),
            );
        }
        return (
            "WITHDRAWAL".to_string(),
            None,
            format!("Amount negative: ${:.2}", tx.amount.abs()),
        );
    }
}

fn contains_any(s: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|p| s.contains(p))
}

fn generate_bank_activity_key(
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
    fn test_classify_salary_deposit() {
        let tx = BankTransactionInput {
            date: "2026-03-25".to_string(),
            description: "EMPLOYER PAYROLL ABC PTY LTD".to_string(),
            amount: 5000.00,
            reference: None,
            category: None,
        };

        let config =
            BankMapperConfig::new("test-account".to_string(), "AUD".to_string(), "BANK_CSV");
        let result = map_single_transaction(&tx, &config);

        assert_eq!(result.activity.activity_type, "DEPOSIT");
        assert_eq!(result.activity.subtype, Some("SALARY".to_string()));
    }

    #[test]
    fn test_classify_shopping_withdrawal() {
        let tx = BankTransactionInput {
            date: "2026-03-25".to_string(),
            description: "WOOLWORTHS 1234 MELBOURNE VIC".to_string(),
            amount: -150.50,
            reference: None,
            category: None,
        };

        let config =
            BankMapperConfig::new("test-account".to_string(), "AUD".to_string(), "BANK_CSV");
        let result = map_single_transaction(&tx, &config);

        assert_eq!(result.activity.activity_type, "WITHDRAWAL");
        assert_eq!(result.activity.subtype, Some("PURCHASE".to_string()));
    }

    #[test]
    fn test_classify_interest_credit() {
        let tx = BankTransactionInput {
            date: "2026-03-31".to_string(),
            description: "INTEREST PAYMENT".to_string(),
            amount: 125.50,
            reference: None,
            category: None,
        };

        let config =
            BankMapperConfig::new("test-account".to_string(), "AUD".to_string(), "BANK_OFX");
        let result = map_single_transaction(&tx, &config);

        assert_eq!(result.activity.activity_type, "INTEREST");
    }

    #[test]
    fn test_classify_fee_debit() {
        let tx = BankTransactionInput {
            date: "2026-03-01".to_string(),
            description: "MONTHLY ACCOUNT FEE".to_string(),
            amount: -10.00,
            reference: None,
            category: None,
        };

        let config =
            BankMapperConfig::new("test-account".to_string(), "AUD".to_string(), "BANK_QIF");
        let result = map_single_transaction(&tx, &config);

        assert_eq!(result.activity.activity_type, "FEE");
    }

    #[test]
    fn test_qif_category_mapping() {
        let tx = BankTransactionInput {
            date: "2026-03-25".to_string(),
            description: "Various items".to_string(),
            amount: 6500.00,
            reference: None,
            category: Some("Salary".to_string()),
        };

        let config =
            BankMapperConfig::new("test-account".to_string(), "AUD".to_string(), "BANK_QIF");
        let result = map_single_transaction(&tx, &config);

        assert_eq!(result.activity.activity_type, "DEPOSIT");
        assert_eq!(result.activity.subtype, Some("SALARY".to_string()));
    }

    #[test]
    fn test_idempotency_key_consistency() {
        let tx = BankTransactionInput {
            date: "2026-03-25".to_string(),
            description: "Test Transaction".to_string(),
            amount: -100.00,
            reference: None,
            category: None,
        };

        let config = BankMapperConfig::new("account-1".to_string(), "AUD".to_string(), "BANK_CSV");
        let result1 = map_single_transaction(&tx, &config);

        let tx2 = BankTransactionInput {
            date: "2026-03-25".to_string(),
            description: "Test Transaction".to_string(),
            amount: -100.00,
            reference: None,
            category: None,
        };
        let result2 = map_single_transaction(&tx2, &config);

        assert_eq!(
            result1.activity.idempotency_key,
            result2.activity.idempotency_key
        );
    }

    #[test]
    fn test_bulk_mapping() {
        let transactions = vec![
            BankTransactionInput {
                date: "2026-03-25".to_string(),
                description: "EMPLOYER PAY".to_string(),
                amount: 5000.00,
                reference: None,
                category: None,
            },
            BankTransactionInput {
                date: "2026-03-26".to_string(),
                description: "WOOLWORTHS".to_string(),
                amount: -150.00,
                reference: None,
                category: None,
            },
        ];

        let config =
            BankMapperConfig::new("test-account".to_string(), "AUD".to_string(), "BANK_CSV");
        let results = map_bank_transactions(transactions, &config);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].activity.activity_type, "DEPOSIT");
        assert_eq!(results[1].activity.activity_type, "WITHDRAWAL");
    }
}
