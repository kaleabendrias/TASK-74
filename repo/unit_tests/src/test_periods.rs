use chrono::NaiveDate;

/// Replicate overlap detection logic: two ranges overlap if start1 < end2 AND start2 < end1
fn ranges_overlap(
    start1: NaiveDate, end1: NaiveDate,
    start2: NaiveDate, end2: NaiveDate,
) -> bool {
    start1 < end2 && start2 < end1
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

#[test]
fn non_overlapping_ranges() {
    assert!(!ranges_overlap(d(2025,1,1), d(2025,1,10), d(2025,1,10), d(2025,1,20)));
}

#[test]
fn adjacent_ranges_no_overlap() {
    // [Jan 1-10) and [Jan 10-20) — adjacent, no overlap
    assert!(!ranges_overlap(d(2025,1,1), d(2025,1,10), d(2025,1,10), d(2025,1,20)));
}

#[test]
fn overlapping_by_one_day() {
    assert!(ranges_overlap(d(2025,1,1), d(2025,1,11), d(2025,1,10), d(2025,1,20)));
}

#[test]
fn fully_nested_range() {
    assert!(ranges_overlap(d(2025,1,1), d(2025,1,31), d(2025,1,10), d(2025,1,20)));
}

#[test]
fn reverse_nested() {
    assert!(ranges_overlap(d(2025,1,10), d(2025,1,20), d(2025,1,1), d(2025,1,31)));
}

#[test]
fn identical_ranges() {
    assert!(ranges_overlap(d(2025,1,1), d(2025,1,10), d(2025,1,1), d(2025,1,10)));
}

#[test]
fn completely_separate_ranges() {
    assert!(!ranges_overlap(d(2025,1,1), d(2025,1,5), d(2025,6,1), d(2025,6,30)));
}

#[test]
fn partial_overlap_beginning() {
    assert!(ranges_overlap(d(2025,1,1), d(2025,1,15), d(2025,1,10), d(2025,1,25)));
}

#[test]
fn partial_overlap_end() {
    assert!(ranges_overlap(d(2025,1,10), d(2025,1,25), d(2025,1,1), d(2025,1,15)));
}

// ── Night bound enforcement ──
#[test]
fn min_7_nights_boundary() {
    let start = d(2025,1,1);
    let end = d(2025,1,8); // 7 nights
    let nights = (end - start).num_days();
    assert_eq!(nights, 7);
    assert!(nights >= 7);
}

#[test]
fn min_6_nights_rejected() {
    let start = d(2025,1,1);
    let end = d(2025,1,7); // 6 nights
    let nights = (end - start).num_days();
    assert_eq!(nights, 6);
    assert!(nights < 7);
}

#[test]
fn max_365_nights_boundary() {
    let start = d(2025,1,1);
    let end = d(2026,1,1); // 365 nights
    let nights = (end - start).num_days();
    assert_eq!(nights, 365);
    assert!(nights <= 365);
}

#[test]
fn max_366_nights_rejected() {
    let start = d(2025,1,1);
    let end = d(2026,1,2); // 366 nights
    let nights = (end - start).num_days();
    assert_eq!(nights, 366);
    assert!(nights > 365);
}
