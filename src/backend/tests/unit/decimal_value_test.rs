// DecimalValue is not currently implemented in the codebase
// These tests are disabled until the feature is added
#![cfg(feature = "bigdecimal")]

use net_ninja::models::types::DecimalValue;
use sqlx::types::BigDecimal;
use std::str::FromStr;

#[test]
fn test_decimal_from_f64() {
    let decimal = DecimalValue::from_f64(123.45);
    let back = decimal.to_f64().unwrap();

    assert!((back - 123.45).abs() < 0.001);
}

#[test]
fn test_decimal_precision() {
    let bd = BigDecimal::from_str("123.456789").unwrap();
    let decimal = DecimalValue::from(bd.clone());

    assert_eq!(decimal.inner().to_string(), "123.456789");
}

#[test]
fn test_decimal_zero() {
    let decimal = DecimalValue::from_f64(0.0);
    assert_eq!(decimal.to_f64().unwrap(), 0.0);
}

#[test]
fn test_decimal_negative() {
    let decimal = DecimalValue::from_f64(-50.25);
    assert_eq!(decimal.to_f64().unwrap(), -50.25);
}
