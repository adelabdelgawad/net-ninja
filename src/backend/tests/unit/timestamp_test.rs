use net_ninja::models::types::Timestamp;
use chrono::{Utc, TimeZone};

#[test]
fn test_timestamp_now() {
    let ts1 = Timestamp::now();
    let ts2 = Timestamp::now();

    assert!(ts2.inner() >= ts1.inner());
}

#[test]
fn test_timestamp_from_datetime() {
    let dt = Utc.with_ymd_and_hms(2024, 1, 21, 12, 30, 0).unwrap();
    let timestamp = Timestamp::from(dt);

    assert_eq!(timestamp.inner(), dt);
}

#[test]
fn test_timestamp_equality() {
    let dt = Utc::now();
    let ts1 = Timestamp::from(dt);
    let ts2 = Timestamp::from(dt);

    assert_eq!(ts1, ts2);
}
