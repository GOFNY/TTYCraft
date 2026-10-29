#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub name: String,
}

impl Account {
    pub fn offline(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

pub const USERNAME_MIN_LEN: usize = 3;
pub const USERNAME_MAX_LEN: usize = 16;

/// Validates an offline username using the same shape as Minecraft accounts
pub fn validate_username(name: &str) -> Result<(), &'static str> {
    let len = name.chars().count();
    if !(USERNAME_MIN_LEN..=USERNAME_MAX_LEN).contains(&len) {
        return Err("Username must be 3-16 characters long.");
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Use only letters, numbers and underscores.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_usernames() {
        assert!(validate_username("Steve").is_ok());
        assert!(validate_username("player_1").is_ok());
        assert!(validate_username("abc").is_ok());
        assert!(validate_username("a234567890123456").is_ok());
    }

    #[test]
    fn rejects_invalid_usernames() {
        assert!(validate_username("").is_err());
        assert!(validate_username("ab").is_err());
        assert!(validate_username("a2345678901234567").is_err());
        assert!(validate_username("steve!").is_err());
        assert!(validate_username("steve space").is_err());
    }
}
