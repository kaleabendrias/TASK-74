use tourism_backend::service::validation;

#[test]
fn deposit_at_exactly_1_5x_ok() {
    // rent=1000, deposit=1500 → exactly 1.5x → should pass
    assert!(validation::validate_deposit_cap(1500.0, 1000.0).is_ok());
}

#[test]
fn deposit_below_cap_ok() {
    assert!(validation::validate_deposit_cap(1000.0, 1000.0).is_ok());
}

#[test]
fn deposit_zero_ok() {
    assert!(validation::validate_deposit_cap(0.0, 1000.0).is_ok());
}

#[test]
fn deposit_exceeds_1_5x_error() {
    let err = validation::validate_deposit_cap(1501.0, 1000.0).unwrap_err();
    assert_eq!(err.body.code, "DEPOSIT_CAP_EXCEEDED");
    assert!(err.body.message.contains("1500.00"));
}

#[test]
fn deposit_at_1_51x_error() {
    let err = validation::validate_deposit_cap(1510.0, 1000.0).unwrap_err();
    assert_eq!(err.body.code, "DEPOSIT_CAP_EXCEEDED");
}

#[test]
fn deposit_tiny_amounts() {
    assert!(validation::validate_deposit_cap(0.15, 0.10).is_ok());
    let err = validation::validate_deposit_cap(0.16, 0.10).unwrap_err();
    assert_eq!(err.body.code, "DEPOSIT_CAP_EXCEEDED");
}

#[test]
fn both_zero_ok() {
    assert!(validation::validate_deposit_cap(0.0, 0.0).is_ok());
}

#[test]
fn large_amounts() {
    assert!(validation::validate_deposit_cap(150_000.0, 100_000.0).is_ok());
    let err = validation::validate_deposit_cap(150_001.0, 100_000.0).unwrap_err();
    assert_eq!(err.body.code, "DEPOSIT_CAP_EXCEEDED");
}
