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
//! once, when the value is created, so inner code can trust it.

use thiserror::Error;

/// The most tickets one user can hold in one event. Decided by Leonardo.
pub const MAX_TICKETS_PER_USER: u32 = 4;

/// A ticket count between 1 and `MAX_TICKETS_PER_USER`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
}
