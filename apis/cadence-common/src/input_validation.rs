use bcrypt::{BcryptError, BcryptResult, verify};
use bcrypt::{DEFAULT_COST, hash};

pub fn is_valid_email(email: &str) -> bool {
    // Consider using a dedicated email validation crate for more robustness if needed
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    re.is_match(email)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordComplexity {
    Flexible,
    Normal,
    Strict,
}

/// Checks password complexity: at least 1 uppercase, 1 lowercase, 1 digit, 1 special char.
pub fn meets_password_complexity(password: &str, complexity: PasswordComplexity) -> bool {
    if complexity == PasswordComplexity::Flexible {
        if password.len() < 6 {
            return false;
        }
    }

    let length = password.len() > 8;
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());

    if complexity == PasswordComplexity::Normal {
        return length && has_lowercase && has_uppercase;
    }

    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| r#"@$!%*?#()/$[]{}&"#.contains(c));
    return length && has_lowercase && has_uppercase && has_digit && has_special;
}

pub fn is_valid_country_code(country_code: &str) -> bool {
    // Check if the country code is exactly 2 characters long and contains only uppercase letters
    country_code.len() == 2 && country_code.chars().all(|c| c.is_ascii_uppercase())
}

pub fn is_valid_name(name: &str) -> bool {
    // Check if the name is not empty and does not exceed 50 characters
    !name.trim().is_empty() && name.len() <= 50
}

pub fn password_to_hashed(password: &str) -> BcryptResult<String> {
    hash(password, DEFAULT_COST)
}

pub fn check_password(password_attempt: &str, stored_hash: &str) -> Result<bool, BcryptError> {
    verify(password_attempt, stored_hash)
}

pub fn string_to_uuid(uuid_str: &str) -> Result<uuid::Uuid, uuid::Error> {
    uuid::Uuid::parse_str(uuid_str)
}
