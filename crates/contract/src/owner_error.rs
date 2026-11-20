// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: B (event_owner, inventory, idempotency), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The failed answers of the `event_owner`.
//!
//! These errors cross areas: area B returns them and area C turns them into
//! HTTP answers, in one place (AI_RULES, section 3). This crate knows nothing
//! about HTTP.
//!
//! A failed answer to `Reserve` or `Buy` is stored under its idempotency key,
//! so a retry with the same key gives the same failure (I4). The one
//! exception is `TemporarilyUnavailable`: nothing was written, so nothing is
//! stored, and the client can retry with the same key. See
//! `OwnerError::is_stored_under_key`.

use crate::reservation_status::ReservationStatus;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Why a command to the owner failed.
// LEARN: `tag = "error"` puts the variant name in a field called "error", for
// example {"error":"sold_out"}. A variant with data adds its fields next to
// it: {"error":"reservation_not_active","status":"expired"}. These codes are
// STORED in the idempotency table. After the contract is frozen, renaming a
// variant would make old stored rows unreadable.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum OwnerError {
    /// The session left before. `Left` is final, so it cannot join again.
    #[error("this session already left the queue and cannot join again")]
    AlreadyLeft,
    /// The session is not in the queue of this event.
    #[error("this session is not in the queue")]
    NotInQueue,
    /// The user is still waiting and has not been admitted.
    #[error("the user is not admitted yet")]
    NotAdmitted,
    /// The user already holds an active reservation for this event.
    #[error("the user already has an active reservation")]
    AlreadyReserved,
    /// The user already bought tickets for this event and cannot reserve again.
    #[error("the user already bought tickets for this event")]
    AlreadyPurchased,
    /// There are no tickets left.
    #[error("sold out")]
    SoldOut,
    /// There are some tickets left, but fewer than the user asked for.
    #[error("not enough tickets left for this quantity")]
    NotEnoughStock,
    /// No reservation with this ID belongs to this user.
    #[error("reservation not found")]
    ReservationNotFound,
    /// The reservation exists but is not `Reserved` any more.
    #[error("reservation is not active, its status is {status:?}")]
    ReservationNotActive { status: ReservationStatus },
    /// The mock payment failed. The reservation stays `Reserved` until it
    /// expires. To try again, the user must send a NEW idempotency key.
    #[error("payment failed")]
    PaymentFailed,
    /// The same idempotency key was used for a different request.
    #[error("idempotency key was already used for a different request")]
    IdempotencyKeyReused,
    /// The owner cannot work now (for example the database is down). Nothing
    /// changed, and the owner keeps running.
    #[error("temporarily unavailable")]
    TemporarilyUnavailable,
}

impl OwnerError {
    // LEARN: This `match` lists every variant on purpose. It has no `_`
    // catch-all. If someone adds a new error later, the compiler stops here
    // and forces a choice: is the new error stored under the key or not?
    // A `_` arm would hide the question, and a wrong answer breaks I4.
    /// True if a failed `Reserve` or `Buy` with this error is stored under
    /// its idempotency key, so a retry with the same key gets the same error.
    ///
    /// Errors: none.
    /// Guarantees: false only for `TemporarilyUnavailable`, because nothing
    /// was written in that case. True for every other error.
    pub fn is_stored_under_key(&self) -> bool {
        match self {
            OwnerError::TemporarilyUnavailable => false,
            OwnerError::AlreadyLeft
            | OwnerError::NotInQueue
            | OwnerError::NotAdmitted
            | OwnerError::AlreadyReserved
            | OwnerError::AlreadyPurchased
            | OwnerError::SoldOut
            | OwnerError::NotEnoughStock
            | OwnerError::ReservationNotFound
            | OwnerError::ReservationNotActive { .. }
            | OwnerError::PaymentFailed
            | OwnerError::IdempotencyKeyReused => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn every_error() -> Vec<OwnerError> {
        vec![
            OwnerError::AlreadyLeft,
            OwnerError::NotInQueue,
            OwnerError::NotAdmitted,
            OwnerError::AlreadyReserved,
            OwnerError::AlreadyPurchased,
            OwnerError::SoldOut,
            OwnerError::NotEnoughStock,
            OwnerError::ReservationNotFound,
            OwnerError::ReservationNotActive {
                status: ReservationStatus::Expired,
            },
            OwnerError::PaymentFailed,
            OwnerError::IdempotencyKeyReused,
            OwnerError::TemporarilyUnavailable,
        ]
    }

    #[test]
    fn every_error_has_its_own_message() {
        let errors = every_error();
        let messages: HashSet<String> = errors.iter().map(|error| error.to_string()).collect();
        assert_eq!(messages.len(), errors.len());
    }

    #[test]
    fn not_active_error_names_the_status() {
        let error = OwnerError::ReservationNotActive {
            status: ReservationStatus::Purchased,
        };
        assert_eq!(
            error.to_string(),
            "reservation is not active, its status is Purchased"
        );
    }

    #[test]
    fn only_the_temporary_failure_is_not_stored() {
        for error in every_error() {
            assert_eq!(
                error.is_stored_under_key(),
                error != OwnerError::TemporarilyUnavailable,
                "{error:?}"
            );
        }
    }

    #[test]
    fn json_form_has_a_stable_error_code() {
        assert_eq!(
            serde_json::to_string(&OwnerError::SoldOut).unwrap(),
            r#"{"error":"sold_out"}"#
        );
        let error = OwnerError::ReservationNotActive {
            status: ReservationStatus::Expired,
        };
        assert_eq!(
            serde_json::to_string(&error).unwrap(),
            r#"{"error":"reservation_not_active","status":"expired"}"#
        );
        for error in every_error() {
            let json = serde_json::to_string(&error).unwrap();
            assert_eq!(serde_json::from_str::<OwnerError>(&json).unwrap(), error);
        }
    }
}
