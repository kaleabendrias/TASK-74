/// Mask a phone number: "(415) ***-1234"
pub fn mask_phone(phone: &str) -> String {
    let digits: Vec<char> = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 10 {
        let area: String = digits[..3].iter().collect();
        let last4: String = digits[digits.len()-4..].iter().collect();
        format!("({}) ***-{}", area, last4)
    } else {
        "***-****".to_string()
    }
}

/// Mask an email: "j***n@example.com"
pub fn mask_email(email: &str) -> String {
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
