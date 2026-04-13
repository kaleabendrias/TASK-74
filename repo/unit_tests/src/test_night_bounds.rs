use chrono::NaiveDate;

fn validate_night_bounds(start: NaiveDate, end: NaiveDate, min: i32, max: i32) -> Result<(), String> {
    if start >= end {
        return Err("start_date must be before end_date".into());
    }
    let nights = (end - start).num_days() as i32;
    if min < 7 {
        return Err("Minimum nights must be at least 7".into());
    }
    if max > 365 {
        return Err("Maximum nights must not exceed 365".into());
    }
    if nights < min {
        return Err(format!("Period is {} nights, minimum is {}", nights, min));
    }
    if nights > max {
        return Err(format!("Period is {} nights, maximum is {}", nights, max));
    }
    Ok(())
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

#[test]
fn valid_7_nights() {
    assert!(validate_night_bounds(d(2025,1,1), d(2025,1,8), 7, 365).is_ok());
}

#[test]
fn valid_365_nights() {
    assert!(validate_night_bounds(d(2025,1,1), d(2026,1,1), 7, 365).is_ok());
}

#[test]
fn min_6_rejected() {
    let err = validate_night_bounds(d(2025,1,1), d(2025,1,8), 6, 365);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("at least 7"));
}

#[test]
fn max_366_rejected() {
    let err = validate_night_bounds(d(2025,1,1), d(2026,1,2), 7, 366);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("exceed 365"));
}

#[test]
fn period_too_short() {
    let err = validate_night_bounds(d(2025,1,1), d(2025,1,5), 7, 365);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("4 nights"));
}

#[test]
fn period_too_long() {
    let err = validate_night_bounds(d(2025,1,1), d(2027,1,1), 7, 365);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("730 nights"));
}

#[test]
fn start_equals_end_rejected() {
    let err = validate_night_bounds(d(2025,1,1), d(2025,1,1), 7, 365);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("before"));
}

#[test]
fn start_after_end_rejected() {
    let err = validate_night_bounds(d(2025,1,10), d(2025,1,1), 7, 365);
    assert!(err.is_err());
}

#[test]
fn exactly_min_boundary() {
    assert!(validate_night_bounds(d(2025,1,1), d(2025,1,8), 7, 365).is_ok());
}

#[test]
fn one_below_min_rejected() {
    // 6 nights with min=7
    let err = validate_night_bounds(d(2025,1,1), d(2025,1,7), 7, 365);
    assert!(err.is_err());
}
