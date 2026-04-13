/// Reimplementation of frontend mask functions for unit testing.
fn mask_phone(phone: &str) -> String {
    let digits: Vec<char> = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 10 {
        let area: String = digits[..3].iter().collect();
        let last4: String = digits[digits.len()-4..].iter().collect();
        format!("({}) ***-{}", area, last4)
    } else {
        "***-****".to_string()
    }
}

fn mask_email(email: &str) -> String {
    match email.split_once('@') {
        Some((local, domain)) => {
            if local.len() <= 2 {
                format!("{}***@{}", &local[..1], domain)
            } else {
                let first = &local[..1];
                let last = &local[local.len()-1..];
                format!("{}***{}@{}", first, last, domain)
            }
        }
        None => "***".to_string(),
    }
}

#[test]
fn phone_standard_10_digit() {
    assert_eq!(mask_phone("4155551234"), "(415) ***-1234");
}

#[test]
fn phone_with_formatting() {
    assert_eq!(mask_phone("(415) 555-1234"), "(415) ***-1234");
}

#[test]
fn phone_with_country_code() {
    assert_eq!(mask_phone("+1-415-555-1234"), "(141) ***-1234");
}

#[test]
fn phone_short_number() {
    assert_eq!(mask_phone("12345"), "***-****");
}

#[test]
fn phone_empty() {
    assert_eq!(mask_phone(""), "***-****");
}

#[test]
fn email_normal() {
    assert_eq!(mask_email("john@example.com"), "j***n@example.com");
}

#[test]
fn email_short_local() {
    assert_eq!(mask_email("ab@example.com"), "a***@example.com");
}

#[test]
fn email_single_char() {
    assert_eq!(mask_email("a@example.com"), "a***@example.com");
}

#[test]
fn email_no_at() {
    assert_eq!(mask_email("noemail"), "***");
}

#[test]
fn email_long_local() {
    assert_eq!(mask_email("longusername@example.com"), "l***e@example.com");
}

#[test]
fn email_with_dots() {
    assert_eq!(mask_email("first.last@example.com"), "f***t@example.com");
}
