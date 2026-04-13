/// Simulate the reserve logic: decrement on_hand, increment reserved, reject if insufficient.
fn try_reserve(on_hand: i32, reserved: i32, qty: i32) -> Result<(i32, i32), &'static str> {
    if qty <= 0 {
        return Err("quantity must be positive");
    }
    if on_hand < qty {
        return Err("insufficient quantity on hand");
    }
    Ok((on_hand - qty, reserved + qty))
}

#[test]
fn reserve_exact_stock() {
    let (oh, res) = try_reserve(10, 0, 10).unwrap();
    assert_eq!(oh, 0);
    assert_eq!(res, 10);
}

#[test]
fn reserve_partial_stock() {
    let (oh, res) = try_reserve(10, 5, 3).unwrap();
    assert_eq!(oh, 7);
    assert_eq!(res, 8);
}

#[test]
fn over_reservation_fails() {
    let err = try_reserve(5, 0, 6).unwrap_err();
    assert_eq!(err, "insufficient quantity on hand");
}

#[test]
fn reserve_zero_fails() {
    let err = try_reserve(10, 0, 0).unwrap_err();
    assert_eq!(err, "quantity must be positive");
}

#[test]
fn reserve_negative_fails() {
    let err = try_reserve(10, 0, -1).unwrap_err();
    assert_eq!(err, "quantity must be positive");
}

#[test]
fn reserve_when_already_reserved() {
    // on_hand=10, reserved=8, trying to reserve 3 more → only 10 on hand
    let (oh, res) = try_reserve(10, 8, 3).unwrap();
    assert_eq!(oh, 7);
    assert_eq!(res, 11);
}

#[test]
fn concurrent_reservations_scenario() {
    // Simulate two sequential reservations against the same initial stock
    let initial_oh = 10;
    let initial_res = 0;
    // First reservation
    let (oh1, res1) = try_reserve(initial_oh, initial_res, 6).unwrap();
    assert_eq!(oh1, 4);
    assert_eq!(res1, 6);
    // Second reservation using updated values
    let (oh2, res2) = try_reserve(oh1, res1, 4).unwrap();
    assert_eq!(oh2, 0);
    assert_eq!(res2, 10);
    // Third reservation should fail
    assert!(try_reserve(oh2, res2, 1).is_err());
}

#[test]
fn reserve_one_unit() {
    let (oh, res) = try_reserve(1, 0, 1).unwrap();
    assert_eq!(oh, 0);
    assert_eq!(res, 1);
}
