#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordValidationError {
    TooShort,
    TooLong,
}

pub fn validate_password(password: &str) -> Result<(), PasswordValidationError> {
    if password.len() < 8 {
        return Err(PasswordValidationError::TooShort);
    }
    if password.len() > 72 {
        return Err(PasswordValidationError::TooLong);
    }
    Ok(())
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, 12)
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_boundaries_are_enforced() {
        assert_eq!(
            validate_password("1234567"),
            Err(PasswordValidationError::TooShort)
        );
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password(&"a".repeat(72)).is_ok());
        assert_eq!(
            validate_password(&"a".repeat(73)),
            Err(PasswordValidationError::TooLong)
        );
    }
}
