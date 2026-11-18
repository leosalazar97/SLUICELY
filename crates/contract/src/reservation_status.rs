// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: B (inventory, store, event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The life of a reservation.
//!
//! `Reserved` becomes `Purchased`, `Expired` or `Cancelled`. The text form
//! (`as_str`, `parse`) is what the database stores, so nobody writes their own.

use thiserror::Error;

// LEARN: A Rust `enum` is like a Java enum, but each value can also carry
// data. Here the values carry nothing. `match` on it must cover every value,
// so if we add a status later, the compiler shows every place to update.
/// The state of one reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReservationStatus {
    /// Tickets are held for the user until `expires_at`.
    Reserved,
    /// The purchase is complete.
    Purchased,
    /// The time ran out and the tickets went back to stock.
    Expired,
    /// The user or the system cancelled it and the tickets went back to stock.
    Cancelled,
}

/// The text is not a known status.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown reservation status: {value}")]
pub struct ReservationStatusParseError {
    value: String,
}

impl ReservationStatus {
    /// Every status, in life order.
    pub const ALL: [ReservationStatus; 4] = [
        ReservationStatus::Reserved,
        ReservationStatus::Purchased,
        ReservationStatus::Expired,
        ReservationStatus::Cancelled,
    ];

    /// Returns the text stored in the database.
    ///
    /// Errors: none.
    /// Guarantees: `parse(status.as_str())` gives back `status`.
    pub fn as_str(&self) -> &'static str {
        match self {
            ReservationStatus::Reserved => "reserved",
            ReservationStatus::Purchased => "purchased",
            ReservationStatus::Expired => "expired",
            ReservationStatus::Cancelled => "cancelled",
        }
    }

    /// Reads a status from the text stored in the database.
    ///
    /// Errors: `ReservationStatusParseError` if the text is not one of the four.
    /// Guarantees: exact match only, no trimming and no changing of case.
    pub fn parse(text: &str) -> Result<Self, ReservationStatusParseError> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == text)
            .ok_or_else(|| ReservationStatusParseError {
                value: text.to_owned(),
            })
    }

    /// True if these tickets count against the stock (`sold + reserved`, I1).
    ///
    /// Errors: none.
    /// Guarantees: true only for `Reserved` and `Purchased`.
    pub fn holds_stock(&self) -> bool {
        matches!(
            self,
            ReservationStatus::Reserved | ReservationStatus::Purchased
        )
    }

    /// True if the status can never change again.
    ///
    /// Errors: none.
    /// Guarantees: true for everything except `Reserved`.
    pub fn is_final(&self) -> bool {
        !matches!(self, ReservationStatus::Reserved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trip_works_for_every_status() {
        for status in ReservationStatus::ALL {
            assert_eq!(ReservationStatus::parse(status.as_str()), Ok(status));
        }
    }

    #[test]
    fn unknown_text_is_rejected_exactly() {
        for bad in ["", "Reserved", " reserved", "sold", "reserved "] {
            assert!(
                ReservationStatus::parse(bad).is_err(),
                "should reject {bad:?}"
            );
        }
    }

    #[test]
    fn stock_and_final_rules_are_exact() {
        let table = [
            (ReservationStatus::Reserved, true, false),
            (ReservationStatus::Purchased, true, true),
            (ReservationStatus::Expired, false, true),
            (ReservationStatus::Cancelled, false, true),
        ];
        for (status, holds_stock, is_final) in table {
            assert_eq!(status.holds_stock(), holds_stock, "{status:?}");
            assert_eq!(status.is_final(), is_final, "{status:?}");
        }
    }
}
