// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: B (inventory, store), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! How many tickets one reservation asks for.
//!
//! A user has one active reservation per event (invariant I2). That
//! reservation holds 1 to `MAX_TICKETS_PER_USER` tickets. The check happens
//! once, when the value is created, so inner code can trust it. The same
//! check runs when the value is read from JSON.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The most tickets one user can hold in one event. Decided by Leonardo.
pub const MAX_TICKETS_PER_USER: u32 = 4;

// LEARN: A plain `derive(Deserialize)` would write the number straight into
// the private field and skip `TicketQuantity::new`. Then JSON with 99 would
// give a quantity of 99, and I1 could break. `try_from = "u32"` tells serde:
// read a `u32` first, then call `TryFrom`, which calls our check. `into =
// "u32"` is the way back when writing JSON. It is like a Jackson
// `@JsonCreator` that validates its input.
/// A ticket count between 1 and `MAX_TICKETS_PER_USER`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct TicketQuantity(u32);

/// Why a quantity was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TicketQuantityError {
    /// The number is 0 or bigger than the maximum.
    #[error("ticket quantity {requested} is not allowed: it must be between 1 and {max}")]
    OutOfRange { requested: u32, max: u32 },
}

impl TicketQuantity {
    /// Checks a number and builds a quantity.
    ///
    /// Errors: `OutOfRange` if the number is 0 or above `MAX_TICKETS_PER_USER`.
    /// Guarantees: a quantity that is returned is between 1 and the maximum.
    pub fn new(value: u32) -> Result<Self, TicketQuantityError> {
        if value == 0 || value > MAX_TICKETS_PER_USER {
            return Err(TicketQuantityError::OutOfRange {
                requested: value,
                max: MAX_TICKETS_PER_USER,
            });
        }
        Ok(Self(value))
    }

    /// Returns the number of tickets.
    ///
    /// Errors: none.
    /// Guarantees: the value is between 1 and `MAX_TICKETS_PER_USER`.
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for TicketQuantity {
    type Error = TicketQuantityError;

    /// Same as `TicketQuantity::new`. Used by serde when it reads JSON.
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<TicketQuantity> for u32 {
    /// Gives the plain number. Used by serde when it writes JSON.
    fn from(quantity: TicketQuantity) -> Self {
        quantity.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_up_to_the_maximum() {
        for value in 1..=MAX_TICKETS_PER_USER {
            assert_eq!(TicketQuantity::new(value).unwrap().get(), value);
        }
    }

    #[test]
    fn rejects_zero() {
        assert_eq!(
            TicketQuantity::new(0),
            Err(TicketQuantityError::OutOfRange {
                requested: 0,
                max: MAX_TICKETS_PER_USER
            })
        );
    }

    #[test]
    fn rejects_one_above_the_maximum() {
        assert!(TicketQuantity::new(MAX_TICKETS_PER_USER + 1).is_err());
        assert!(TicketQuantity::new(u32::MAX).is_err());
    }

    #[test]
    fn json_form_is_a_plain_number_and_is_checked() {
        let quantity = TicketQuantity::new(3).unwrap();
        assert_eq!(serde_json::to_string(&quantity).unwrap(), "3");
        assert_eq!(
            serde_json::from_str::<TicketQuantity>("4").unwrap().get(),
            4
        );
        for bad in ["0", "5", "4294967295", "-1"] {
            assert!(
                serde_json::from_str::<TicketQuantity>(bad).is_err(),
                "should reject {bad}"
            );
        }
    }
}
