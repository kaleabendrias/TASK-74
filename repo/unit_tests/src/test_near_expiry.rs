use chrono::{NaiveDate, Utc};

fn is_near_expiry(expiration_date: Option<NaiveDate>) -> bool {
    expiration_date.map_or(false, |d| {
        let cutoff = Utc::now().date_naive() + chrono::Duration::days(30);
        d <= cutoff
    })
}

fn today_plus(days: i64) -> NaiveDate {
    Utc::now().date_naive() + chrono::Duration::days(days)
}

#[test]
fn exactly_30_days_is_near_expiry() {
    assert!(is_near_expiry(Some(today_plus(30))));
}

#[test]
fn at_29_days_is_near_expiry() {
    assert!(is_near_expiry(Some(today_plus(29))));
}

#[test]
fn at_31_days_is_not_near_expiry() {
    assert!(!is_near_expiry(Some(today_plus(31))));
}

#[test]
fn already_expired_is_near_expiry() {
    assert!(is_near_expiry(Some(today_plus(-1))));
}

#[test]
fn expired_yesterday() {
    let yesterday = Utc::now().date_naive() - chrono::Duration::days(1);
    assert!(is_near_expiry(Some(yesterday)));
}

#[test]
fn no_expiration_not_near_expiry() {
    assert!(!is_near_expiry(None));
}

#[test]
fn today_is_near_expiry() {
    assert!(is_near_expiry(Some(Utc::now().date_naive())));
}

#[test]
fn far_future_not_near_expiry() {
    assert!(!is_near_expiry(Some(today_plus(365))));
}
