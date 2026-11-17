// ============================================================
// SHARED FILE
// Owner: contract (written by Claude). Others: do not edit. Ask Leonardo.
// Used by: B (idempotency, store), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The idempotency key.
//!
//! The client sends a key with reserve and buy. The same key must give the
//! same result (invariant I4). This type checks the key once, at the edge, so
//! the inner modules can trust it.

use thiserror::Error;

/// The longest key we accept, in bytes. This is a proposal, not a measured
/// value. A UUID is 36 characters, so there is a lot of room.
pub const MAX_IDEMPOTENCY_KEY_LENGTH: usize = 128;

/// An idempotency key that passed the checks of `IdempotencyKey::parse`.
///
/// It is never changed after creation (no trimming, no lower-casing), because
/// a change would turn two different client keys into one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdempotencyKey(String);

// LEARN: `thiserror` writes the boring error code for you. `#[derive(Error)]`
// plus `#[error("...")]` gives each variant a readable message. In Java this
// is like a custom exception class with a message, but here the compiler
// forces the caller to handle every variant. The `api` crate will turn these
// errors into an HTTP 400 answer. This crate knows nothing about HTTP.
/// Why a key was rejected.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IdempotencyKeyError {
    /// The key has no characters.
    #[error("idempotency key is empty")]
    Empty,
    /// The key has a character that is not visible ASCII (space, control, or non-ASCII).
    #[error("idempotency key has a character that is not visible ASCII")]
    InvalidCharacter,
    /// The key is longer than `MAX_IDEMPOTENCY_KEY_LENGTH`.
    #[error("idempotency key is too long: {actual} bytes, the maximum is {max}")]
    TooLong { actual: usize, max: usize },
}

impl IdempotencyKey {
    // LEARN: `chars().find(...)` walks over the characters and returns the
    // first one that matches, or `None`. Like `stream().filter().findFirst()`
    // in Java. `is_ascii_graphic` means a visible ASCII character: letters,
    // digits and punctuation, but no space and no control characters. This
    // also blocks new lines, so a client cannot fake a line in our logs.
    /// Checks a raw text and builds a key.
    ///
    /// Errors: `Empty`, `InvalidCharacter`, `TooLong`. The checks run in this
    /// order.
    /// Guarantees: a key that is returned is not empty, has only visible
    /// ASCII characters, and is at most `MAX_IDEMPOTENCY_KEY_LENGTH` bytes.
    pub fn parse(raw: &str) -> Result<Self, IdempotencyKeyError> {
        if raw.is_empty() {
            return Err(IdempotencyKeyError::Empty);
        }
        if raw.chars().any(|character| !character.is_ascii_graphic()) {
            return Err(IdempotencyKeyError::InvalidCharacter);
        }
        // Here all characters are ASCII, so bytes and characters are the same.
        if raw.len() > MAX_IDEMPOTENCY_KEY_LENGTH {
            return Err(IdempotencyKeyError::TooLong {
                actual: raw.len(),
                max: MAX_IDEMPOTENCY_KEY_LENGTH,
            });
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the key as text.
    ///
    /// Errors: none.
    /// Guarantees: returns exactly the text that `parse` accepted.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Display shows the key. Logs use it: `info!(idempotency_key = %key, ...)`.
impl std::fmt::Display for IdempotencyKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_normal_key_and_keeps_it_unchanged() {
        let key = IdempotencyKey::parse("3f2b8c1e-7d4a-4b6e-9a10-2c5d8e7f1a90").unwrap();
        assert_eq!(key.as_str(), "3f2b8c1e-7d4a-4b6e-9a10-2c5d8e7f1a90");
        assert_eq!(key.to_string(), key.as_str());
    }

    #[test]
    fn rejects_an_empty_key() {
        assert_eq!(IdempotencyKey::parse(""), Err(IdempotencyKeyError::Empty));
    }

    #[test]
    fn rejects_space_new_line_and_non_ascii() {
        for bad in ["a b", "a\nb", " ", "caffè"] {
            assert_eq!(
                IdempotencyKey::parse(bad),
                Err(IdempotencyKeyError::InvalidCharacter),
                "should reject {bad:?}"
            );
        }
    }

    #[test]
    fn length_limit_is_exact() {
        let longest_ok = "a".repeat(MAX_IDEMPOTENCY_KEY_LENGTH);
        assert!(IdempotencyKey::parse(&longest_ok).is_ok());

        let too_long = "a".repeat(MAX_IDEMPOTENCY_KEY_LENGTH + 1);
        assert_eq!(
            IdempotencyKey::parse(&too_long),
            Err(IdempotencyKeyError::TooLong {
                actual: MAX_IDEMPOTENCY_KEY_LENGTH + 1,
                max: MAX_IDEMPOTENCY_KEY_LENGTH,
            })
        );
    }

    #[test]
    fn different_text_gives_different_keys() {
        // No hidden changes: "Key" and "key" are two different keys.
        let upper = IdempotencyKey::parse("Key").unwrap();
        let lower = IdempotencyKey::parse("key").unwrap();
        assert_ne!(upper, lower);
    }
}
