// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, controller), B (event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! A place in the queue. Place 1 is the first user to be served.
//!
//! The JSON form is a plain number. Reading JSON runs the same check as
//! `QueuePosition::new`, so place 0 can never be built.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// LEARN: Same idea as in `TicketQuantity`. `try_from = "u64"` makes serde
// read a number and then call our `TryFrom`, which rejects 0. Without it,
// JSON with 0 would build an invalid position.
/// A queue place, counted from 1. Place 0 does not exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct QueuePosition(u64);

/// Why a position was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum QueuePositionError {
    /// The number is 0. Places start at 1.
    #[error("queue position must be 1 or more")]
    Zero,
    /// The number is bigger than `i64::MAX`. The database column is `bigint`.
    #[error("queue position is too large for the database")]
    TooLarge,
}

impl QueuePosition {
    /// Builds a position from a number counted from 1.
    ///
    /// Errors: none. It returns `None` for 0 and for a number above
    /// `i64::MAX`, so the caller must handle it.
    /// Guarantees: a position that is returned is between 1 and `i64::MAX`.
    pub fn new(value: u64) -> Option<Self> {
        if value == 0 || value > i64::MAX as u64 {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Returns the place number, counted from 1.
    ///
    /// Errors: none.
    /// Guarantees: the value is between 1 and `i64::MAX`.
    pub fn get(&self) -> u64 {
        self.0
    }

    /// Gives the place as an `i64`, to write it in a `bigint` column.
    ///
    /// Errors: none.
    /// Guarantees: the value is between 1 and `i64::MAX`. The cast cannot
    /// lose bits, because `new` rejects anything above `i64::MAX`.
    pub fn to_i64(&self) -> i64 {
        self.0 as i64
    }

    /// Builds a position from a `bigint` column.
    ///
    /// Errors: none. It returns `None` for 0 or a negative number, so the
    /// caller must handle it (a bad row in the database).
    /// Guarantees: a position that is returned is between 1 and `i64::MAX`.
    pub fn from_i64(value: i64) -> Option<Self> {
        if value < 1 {
            None
        } else {
            Some(Self(value as u64))
        }
    }
}

impl TryFrom<u64> for QueuePosition {
    type Error = QueuePositionError;

    /// Same check as `QueuePosition::new`. Used by serde when it reads JSON.
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value == 0 {
            Err(QueuePositionError::Zero)
        } else {
            Self::new(value).ok_or(QueuePositionError::TooLarge)
        }
    }
}

impl From<QueuePosition> for u64 {
    /// Gives the plain number. Used by serde when it writes JSON.
    fn from(position: QueuePosition) -> Self {
        position.0
    }
}

impl std::fmt::Display for QueuePosition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_not_a_position() {
        assert_eq!(QueuePosition::new(0), None);
    }

    #[test]
    fn keeps_its_number_orders_and_displays() {
        let first = QueuePosition::new(1).unwrap();
        let second = QueuePosition::new(2).unwrap();
        assert_eq!(first.get(), 1);
        assert!(first < second);
        assert_eq!(second.to_string(), "2");
    }

    #[test]
    fn json_form_is_a_plain_number_and_zero_is_rejected() {
        let position = QueuePosition::new(7).unwrap();
        assert_eq!(serde_json::to_string(&position).unwrap(), "7");
        assert_eq!(
            serde_json::from_str::<QueuePosition>("7").unwrap(),
            position
        );
        assert!(serde_json::from_str::<QueuePosition>("0").is_err());
        assert!(serde_json::from_str::<QueuePosition>("-1").is_err());
    }

    #[test]
    fn the_largest_database_value_is_the_maximum() {
        let largest = QueuePosition::new(i64::MAX as u64).unwrap();
        assert_eq!(largest.to_i64(), i64::MAX);
        assert_eq!(QueuePosition::new(i64::MAX as u64 + 1), None);
        assert_eq!(
            QueuePosition::try_from(u64::MAX),
            Err(QueuePositionError::TooLarge)
        );
        assert_eq!(QueuePosition::try_from(0), Err(QueuePositionError::Zero));
        assert!(serde_json::from_str::<QueuePosition>("9223372036854775808").is_err());
    }

    #[test]
    fn database_conversion_round_trips_and_rejects_bad_rows() {
        for value in [1_i64, 2, 1_000, i64::MAX] {
            let position = QueuePosition::from_i64(value).unwrap();
            assert_eq!(position.to_i64(), value);
        }
        assert_eq!(QueuePosition::from_i64(0), None);
        assert_eq!(QueuePosition::from_i64(-1), None);
        assert_eq!(QueuePosition::from_i64(i64::MIN), None);
    }
}
