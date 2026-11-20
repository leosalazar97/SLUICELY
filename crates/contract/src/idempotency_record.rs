// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: B (idempotency, store, event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! What the idempotency table stores (invariant I4).
//!
//! For each key the table remembers the REQUEST (what the client asked) and
//! the RESPONSE (what we answered). On a retry with the same key:
//! - same request: answer with the stored response;
//! - different request: answer `OwnerError::IdempotencyKeyReused`.
//!
//! Without the stored request, the second case could not be detected.

use crate::ids::{ReservationId, UserId};
use crate::owner_answer::{PurchaseAnswer, ReservationAnswer};
use crate::owner_error::OwnerError;
use crate::ticket_quantity::TicketQuantity;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The two operations that carry an idempotency key.
///
/// The text form is the value of the `operation` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdempotentOperation {
    Reserve,
    Buy,
}

/// The text is not a known operation.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown idempotent operation: {value}")]
pub struct IdempotentOperationParseError {
    value: String,
}

impl IdempotentOperation {
    /// Every operation.
    pub const ALL: [IdempotentOperation; 2] =
        [IdempotentOperation::Reserve, IdempotentOperation::Buy];

    /// Returns the text stored in the database.
    ///
    /// Errors: none.
    /// Guarantees: `parse(operation.as_str())` gives back `operation`.
    pub fn as_str(&self) -> &'static str {
        match self {
            IdempotentOperation::Reserve => "reserve",
            IdempotentOperation::Buy => "buy",
        }
    }

    /// Reads an operation from the text stored in the database.
    ///
    /// Errors: `IdempotentOperationParseError` if the text is not known.
    /// Guarantees: exact match only, no trimming and no changing of case.
    pub fn parse(text: &str) -> Result<Self, IdempotentOperationParseError> {
        Self::ALL
            .into_iter()
            .find(|operation| operation.as_str() == text)
            .ok_or_else(|| IdempotentOperationParseError {
                value: text.to_owned(),
            })
    }
}

// LEARN: Two requests are "the same" only if every field is equal. The
// derived `PartialEq` compares all fields, so the owner can write
// `stored_request == new_request`. The `user_id` is part of the request on
// purpose: another user who sends the same key text gets a different
// request, so they never read the response of the first user.
/// What the client asked, in the form we compare on a retry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum IdempotentRequest {
    Reserve {
        user_id: UserId,
        quantity: TicketQuantity,
    },
    Buy {
        user_id: UserId,
        reservation_id: ReservationId,
    },
}

impl IdempotentRequest {
    /// The operation of this request.
    ///
    /// Errors: none.
    /// Guarantees: equal to the `operation` field in the JSON form.
    pub fn operation(&self) -> IdempotentOperation {
        match self {
            IdempotentRequest::Reserve { .. } => IdempotentOperation::Reserve,
            IdempotentRequest::Buy { .. } => IdempotentOperation::Buy,
        }
    }

    /// The user who asked. Errors: none.
    pub fn user_id(&self) -> UserId {
        match self {
            IdempotentRequest::Reserve { user_id, .. } | IdempotentRequest::Buy { user_id, .. } => {
                *user_id
            }
        }
    }
}

// LEARN: `tag = "outcome", content = "response"` writes two fields:
// {"outcome":"failed","response":{"error":"sold_out"}}. The first says which
// variant it is, and the second holds its data. serde calls this "adjacent
// tagging". We use it here because `OwnerError` already has its own tag, and
// two tags in one object would be confusing.
/// The answer that the table stores for a key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", content = "response", rename_all = "snake_case")]
pub enum StoredResponse {
    /// A reservation was created.
    Reserved(ReservationAnswer),
    /// A purchase was completed.
    Purchased(PurchaseAnswer),
    /// The command failed with an error that is stored.
    Failed(OwnerError),
}

impl StoredResponse {
    /// Builds a stored failure.
    ///
    /// Errors: none. It returns `None` if the error must NOT be stored (see
    /// `OwnerError::is_stored_under_key`), so the caller must handle it.
    /// Guarantees: a returned value holds an error that is stored.
    pub fn from_failure(error: OwnerError) -> Option<Self> {
        if error.is_stored_under_key() {
            Some(StoredResponse::Failed(error))
        } else {
            None
        }
    }

    /// True if this response can belong to a row of this operation.
    ///
    /// Errors: none.
    /// Guarantees: a reservation answer fits only `Reserve`, a purchase
    /// answer fits only `Buy`, and a failure fits both.
    pub fn fits(&self, operation: IdempotentOperation) -> bool {
        match self {
            StoredResponse::Reserved(_) => operation == IdempotentOperation::Reserve,
            StoredResponse::Purchased(_) => operation == IdempotentOperation::Buy,
            StoredResponse::Failed(_) => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn user() -> UserId {
        UserId::from_uuid(Uuid::from_u128(1))
    }

    fn reservation() -> ReservationAnswer {
        ReservationAnswer {
            reservation_id: ReservationId::from_uuid(Uuid::from_u128(2)),
            quantity: TicketQuantity::new(2).unwrap(),
            expires_at: DateTime::<Utc>::from_timestamp(1300, 0).unwrap(),
        }
    }

    fn purchase() -> PurchaseAnswer {
        PurchaseAnswer {
            order_id: crate::ids::OrderId::from_uuid(Uuid::from_u128(3)),
            reservation_id: ReservationId::from_uuid(Uuid::from_u128(2)),
            quantity: TicketQuantity::new(2).unwrap(),
        }
    }

    #[test]
    fn operation_text_matches_json_and_parse() {
        for operation in IdempotentOperation::ALL {
            let json = serde_json::to_string(&operation).unwrap();
            assert_eq!(json, format!("\"{}\"", operation.as_str()));
            assert_eq!(
                IdempotentOperation::parse(operation.as_str()),
                Ok(operation)
            );
        }
        assert!(IdempotentOperation::parse("Reserve").is_err());
        assert!(IdempotentOperation::parse("").is_err());
    }

    #[test]
    fn request_json_is_exact_and_tag_equals_operation() {
        let reserve = IdempotentRequest::Reserve {
            user_id: user(),
            quantity: TicketQuantity::new(2).unwrap(),
        };
        let buy = IdempotentRequest::Buy {
            user_id: user(),
            reservation_id: ReservationId::from_uuid(Uuid::from_u128(2)),
        };
        assert_eq!(
            serde_json::to_string(&reserve).unwrap(),
            r#"{"operation":"reserve","user_id":"00000000-0000-0000-0000-000000000001","quantity":2}"#
        );
        assert_eq!(
            serde_json::to_string(&buy).unwrap(),
            r#"{"operation":"buy","user_id":"00000000-0000-0000-0000-000000000001","reservation_id":"00000000-0000-0000-0000-000000000002"}"#
        );
        for request in [reserve, buy] {
            let json = serde_json::to_string(&request).unwrap();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["operation"], request.operation().as_str());
            assert_eq!(
                serde_json::from_str::<IdempotentRequest>(&json).unwrap(),
                request
            );
            assert_eq!(request.user_id(), user());
        }
    }

    #[test]
    fn a_different_request_is_not_equal() {
        let two = IdempotentRequest::Reserve {
            user_id: user(),
            quantity: TicketQuantity::new(2).unwrap(),
        };
        let four = IdempotentRequest::Reserve {
            user_id: user(),
            quantity: TicketQuantity::new(4).unwrap(),
        };
        let other_user = IdempotentRequest::Reserve {
            user_id: UserId::from_uuid(Uuid::from_u128(9)),
            quantity: TicketQuantity::new(2).unwrap(),
        };
        assert_ne!(two, four);
        assert_ne!(two, other_user);
        assert_eq!(two, two);
    }

    #[test]
    fn stored_response_round_trips_and_failure_json_is_exact() {
        let failed = StoredResponse::Failed(OwnerError::SoldOut);
        assert_eq!(
            serde_json::to_string(&failed).unwrap(),
            r#"{"outcome":"failed","response":{"error":"sold_out"}}"#
        );
        for response in [
            StoredResponse::Reserved(reservation()),
            StoredResponse::Purchased(purchase()),
            failed,
        ] {
            let json = serde_json::to_string(&response).unwrap();
            assert_eq!(
                serde_json::from_str::<StoredResponse>(&json).unwrap(),
                response
            );
        }
    }

    #[test]
    fn only_stored_failures_can_be_built() {
        assert_eq!(
            StoredResponse::from_failure(OwnerError::SoldOut),
            Some(StoredResponse::Failed(OwnerError::SoldOut))
        );
        assert_eq!(
            StoredResponse::from_failure(OwnerError::TemporarilyUnavailable),
            None
        );
    }

    #[test]
    fn response_fits_only_its_own_operation() {
        let table = [
            (StoredResponse::Reserved(reservation()), true, false),
            (StoredResponse::Purchased(purchase()), false, true),
            (StoredResponse::Failed(OwnerError::SoldOut), true, true),
        ];
        for (response, fits_reserve, fits_buy) in table {
            assert_eq!(response.fits(IdempotentOperation::Reserve), fits_reserve);
            assert_eq!(response.fits(IdempotentOperation::Buy), fits_buy);
        }
    }
}
