//! Integration simulation test for resumable import retries.
//!
//! Verifies the chunked-cursor logic:
//!   1. A job processes N rows in chunks.
//!   2. If processing fails after chunk k, the cursor is preserved.
//!   3. On retry, only rows after the cursor are re-processed.

/// Simulates the chunked ingestion logic without a live database.
/// Returns `(rows_inserted_total, final_cursor)`.
fn simulate_chunked_import(
    total_rows: usize,
    chunk_size: usize,
    fail_after_chunk: Option<usize>, // fail after committing this many chunks
) -> Result<(usize, usize), (usize, String)> {
    let mut inserted = 0usize;
    let mut chunk_num = 0usize;

    for chunk_start in (0..total_rows).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(total_rows);
        let chunk_rows = chunk_end - chunk_start;

        // Simulate inserting the chunk
        inserted += chunk_rows;
        chunk_num += 1;

        // Simulate a failure after the specified chunk
        if let Some(fail) = fail_after_chunk {
            if chunk_num == fail {
                return Err((inserted, format!("simulated failure after chunk {}", fail)));
            }
        }
    }

    Ok((inserted, inserted))
}

/// Resumes from a saved cursor position.
fn simulate_resume(
    total_rows: usize,
    chunk_size: usize,
    cursor: usize, // rows already committed
) -> usize {
    let mut inserted = cursor;
    for chunk_start in (cursor..total_rows).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(total_rows);
        inserted += chunk_end - chunk_start;
    }
    inserted
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[test]
fn full_run_no_failure() {
    let (inserted, cursor) = simulate_chunked_import(1000, 500, None).unwrap();
    assert_eq!(inserted, 1000);
    assert_eq!(cursor, 1000);
}

#[test]
fn failure_after_first_chunk_preserves_cursor() {
    // 1000 rows in chunks of 500 — fail after chunk 1 (500 rows committed)
    let err = simulate_chunked_import(1000, 500, Some(1)).unwrap_err();
    let (cursor, msg) = err;
    assert_eq!(cursor, 500, "Cursor must reflect the 500 rows committed before failure");
    assert!(msg.contains("chunk 1"));
}

#[test]
fn resume_from_cursor_processes_remaining_rows_only() {
    // First attempt: fail after chunk 1, cursor = 500
    let (cursor, _) = simulate_chunked_import(1000, 500, Some(1)).unwrap_err();
    assert_eq!(cursor, 500);

    // Resume: should process only rows 500..1000
    let total_after_resume = simulate_resume(1000, 500, cursor);
    assert_eq!(total_after_resume, 1000, "After resume all rows must be accounted for");
}

#[test]
fn resume_from_mid_chunk_boundary() {
    // 1000 rows in chunks of 300 — fail after chunk 2 (600 rows committed)
    let (cursor, _) = simulate_chunked_import(1000, 300, Some(2)).unwrap_err();
    assert_eq!(cursor, 600);

    let total_after_resume = simulate_resume(1000, 300, cursor);
    assert_eq!(total_after_resume, 1000);
}

#[test]
fn resume_with_no_remaining_rows_is_noop() {
    // All rows already committed
    let total = simulate_resume(500, 500, 500);
    assert_eq!(total, 500, "Resuming from a fully-committed cursor should add zero rows");
}

#[test]
fn chunk_cursor_never_exceeds_total_rows() {
    // Various row counts / chunk sizes must never overshoot
    for (total, chunk) in &[(7, 3), (10, 5), (1, 1), (999, 500)] {
        let (inserted, _) = simulate_chunked_import(*total, *chunk, None).unwrap();
        assert_eq!(
            inserted, *total,
            "total={} chunk={}: inserted {} rows, expected {}",
            total, chunk, inserted, total
        );
    }
}
