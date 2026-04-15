//! PII masking utilities — single source of truth for both the frontend
//! (`services/mask.rs` delegates here) and `frontend_tests`.

/// Mask a phone number: keeps the area code and last 4 digits.
/// "(415) 555-1234" → "(415) ***-1234"
/// Any input with fewer than 10 digits returns "***-****".
pub fn mask_phone(phone: &str) -> String {
    let digits: Vec<char> = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 10 {
        let area:  String = digits[..3].iter().collect();
        let last4: String = digits[digits.len()-4..].iter().collect();
        format!("({}) ***-{}", area, last4)
    } else {
        "***-****".to_string()
    }
}

/// Mask an email address: keeps the first and last character of the local part.
/// "john@example.com" → "j***n@example.com"
/// Single-character local part: "a@b.com" → "a***@b.com"
/// No `@` present: returns "***".
pub fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => {
            if local.len() <= 2 {
                format!("{}***@{}", &local[..1], domain)
            } else {
                let first = &local[..1];
                let last  = &local[local.len()-1..];
                format!("{}***{}@{}", first, last, domain)
            }
        }
        None => "***".to_string(),
    }
}
