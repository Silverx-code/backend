use lazy_static::lazy_static;
use regex::Regex;
use validator::ValidationError;

lazy_static! {
    pub static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    pub static ref PASSWORD_UPPERCASE: Regex = Regex::new(r"[A-Z]").unwrap();
    pub static ref PASSWORD_LOWERCASE: Regex = Regex::new(r"[a-z]").unwrap();
    pub static ref PASSWORD_NUMBER: Regex = Regex::new(r"[0-9]").unwrap();
    pub static ref PASSWORD_SPECIAL: Regex = Regex::new(r"[!@#$%^&*(),.?:{}|<>]").unwrap();
}

const ALLOWED_EMAIL_DOMAIN: &str = "@undergraduate.mcu.edu.ng";

/// Validates that email ends with @undergraduate.mcu.edu.ng
pub fn validate_mcu_email(email: &str) -> Result<(), ValidationError> {
    if !email.to_lowercase().ends_with(ALLOWED_EMAIL_DOMAIN) {
        let mut error = ValidationError::new("invalid_domain");
        error.message = Some(format!("Email must end with {}", ALLOWED_EMAIL_DOMAIN).into());
        return Err(error);
    }
    Ok(())
}

/// Validates password strength
/// Requirements:
/// - At least 8 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one number
/// - At least one special character
pub fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let mut errors = Vec::new();

    if password.len() < 8 {
        errors.push("Password must be at least 8 characters long");
    }

    if !PASSWORD_UPPERCASE.is_match(password) {
        errors.push("Password must contain at least one uppercase letter");
    }

    if !PASSWORD_LOWERCASE.is_match(password) {
        errors.push("Password must contain at least one lowercase letter");
    }

    if !PASSWORD_NUMBER.is_match(password) {
        errors.push("Password must contain at least one number");
    }

    if !PASSWORD_SPECIAL.is_match(password) {
        errors.push("Password must contain at least one special character (!@#$%^&*(),.?:{}|<>)");
    }

    if !errors.is_empty() {
        let mut error = ValidationError::new("weak_password");
        error.message = Some(errors.join("; ").into());
        return Err(error);
    }

    Ok(())
}