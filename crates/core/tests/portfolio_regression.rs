//! Portfolio snapshot regression tests.
//!
//! Uses `insta` — the Rust snapshot library used by cargo-semver-checks,
//! axum, tracing, and others. Pattern: pin serialised output of critical
//! financial data. Any change surfaces as a reviewable diff.
//!
//! # Updating snapshots
//! ```sh
//! cargo insta test --workspace --review
//! ```

use chrono::Utc;
use insta::{assert_json_snapshot, assert_snapshot};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sensible_folio_core::activities::{Activity, ActivityStatus};
use sensible_folio_core::portfolio::snapshot::AccountStateSnapshot;

fn make_activity(activity_type: &str, qty: Decimal, price: Decimal, fee: Decimal) -> Activity {
    let now = Utc::now();
    Activity {
        id: "test-id".into(),
        account_id: "acc-001".into(),
        asset_id: Some("AAPL".into()),
        activity_type: activity_type.into(),
        activity_type_override: None,
        source_type: None,
        subtype: None,
        status: ActivityStatus::Posted,
        activity_date: now,
        settlement_date: None,
        quantity: Some(qty),
        unit_price: Some(price),
        amount: None,
        fee: Some(fee),
        currency: "USD".into(),
        fx_rate: None,
        notes: None,
        metadata: None,
        source_system: None,
        source_record_id: None,
        source_group_id: None,
        idempotency_key: None,
        import_run_id: None,
        is_user_modified: false,
        needs_review: false,
        created_at: now,
        updated_at: now,
    }
}

// ─── Activity method regressions ────────────────────────────────────────────

/// qty() must return the Decimal without rounding
#[test]
fn regression_activity_qty_parse() {
    let a = make_activity("BUY", dec!(10.5), dec!(150.25), dec!(9.99));
    assert_snapshot!("activity_qty", format!("{}", a.qty()));
}

/// price() must return correct Decimal
#[test]
fn regression_activity_price_parse() {
    let a = make_activity("BUY", dec!(10), dec!(150.255), dec!(0));
    assert_snapshot!("activity_price", format!("{}", a.price()));
}

/// fee_amt() must return the fee field
#[test]
fn regression_activity_fee_parse() {
    let a = make_activity("BUY", dec!(10), dec!(150.00), dec!(9.99));
    assert_snapshot!("activity_fee", format!("{}", a.fee_amt()));
}

/// BUY net cost = qty * price + fee
#[test]
fn regression_buy_net_cost() {
    let a = make_activity("BUY", dec!(10), dec!(150.25), dec!(9.99));
    let net_cost = a.qty() * a.price() + a.fee_amt();
    assert_snapshot!("buy_net_cost_usd", format!("{:.4}", net_cost));
}

/// SELL net proceeds = qty * price − fee
#[test]
fn regression_sell_net_proceeds() {
    let a = make_activity("SELL", dec!(5), dec!(200.00), dec!(4.95));
    let proceeds = a.qty() * a.price() - a.fee_amt();
    assert_snapshot!("sell_net_proceeds_usd", format!("{:.4}", proceeds));
}

/// Activity JSON serialisation must remain stable across refactors
#[test]
fn regression_activity_json_shape() {
    let now = Utc::now();
    let a = Activity {
        id: "regression-001".into(),
        account_id: "acc-001".into(),
        asset_id: Some("MSFT".into()),
        activity_type: "DIVIDEND".into(),
        activity_type_override: None,
        source_type: None,
        subtype: None,
        status: ActivityStatus::Posted,
        activity_date: now,
        settlement_date: None,
        quantity: None,
        unit_price: None,
        amount: Some(dec!(42.50)),
        fee: Some(dec!(0)),
        currency: "USD".into(),
        fx_rate: None,
        notes: None,
        metadata: None,
        source_system: None,
        source_record_id: None,
        source_group_id: None,
        idempotency_key: None,
        import_run_id: None,
        is_user_modified: false,
        needs_review: false,
        created_at: now,
        updated_at: now,
    };

    assert_json_snapshot!("dividend_activity_shape", a, {
        ".activityDate" => "[datetime]",
        ".createdAt" => "[datetime]",
        ".updatedAt" => "[datetime]",
    });
}

// ─── Empty snapshot shape regression ────────────────────────────────────────

/// The shape of AccountStateSnapshot JSON must not change without review.
#[test]
fn regression_empty_snapshot_json_shape() {
    let snap = AccountStateSnapshot {
        id: "00000000-0000-0000-0000-000000000001".into(),
        account_id: "acc-001".into(),
        snapshot_date: "2025-01-01".parse().unwrap(),
        currency: "USD".into(),
        net_contribution: dec!(5000),
        cost_basis: dec!(4800),
        ..Default::default()
    };

    assert_json_snapshot!("empty_snapshot_shape", snap, {
        ".calculatedAt" => "[datetime]",
    });
}
