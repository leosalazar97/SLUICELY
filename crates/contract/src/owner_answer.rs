// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: B (event_owner, idempotency), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The successful answers of the `event_owner`.
//!
//! Join gives a `QueueState`, reserve gives a `ReservationAnswer`, buy gives a
//! `PurchaseAnswer`. Leave has no data, so it answers with `()`. The failed
//! answers are in `OwnerError`. Every value inside is already checked, so the
//! fields are public. The JSON form is used by the api and by the
//! idempotency table. Reading JSON runs the checks of the inner types.

use crate::ids::{OrderId, ReservationId};
use crate::queue_position::QueuePosition;
use crate::ticket_quantity::TicketQuantity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// LEARN: `tag = "state"` writes the variant name in a field "state", for
// example {"state":"waiting","position":5} and {"state":"admitted"}. It is
// easy to read in JavaScript: `if (message.state === "waiting")`.
/// Where a user is in the queue right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum QueueState {
    /// Waiting, at this place.
    Waiting { position: QueuePosition },
    /// Admitted: the user may reserve.
    Admitted,
    /// Joined while the opening window is open. The order is not decided yet,
    /// so there is no position. It is decided when the window closes.
    WindowOpen,
}

/// A reservation that was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReservationAnswer {
    pub reservation_id: ReservationId,
    pub quantity: TicketQuantity,
    /// After this time (UTC) the tickets go back to stock.
    pub expires_at: DateTime<Utc>,
}

/// A purchase that was completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseAnswer {
    pub order_id: OrderId,
    pub reservation_id: ReservationId,
    pub quantity: TicketQuantity,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn answers_keep_their_values() {
        let reservation_id = ReservationId::from_uuid(Uuid::from_u128(1));
        let quantity = TicketQuantity::new(3).unwrap();
        let expires_at = DateTime::<Utc>::from_timestamp(1300, 0).unwrap();

        let reservation = ReservationAnswer {
            reservation_id,
            quantity,
            expires_at,
        };
        assert_eq!(reservation.quantity.get(), 3);
        assert_eq!(reservation.expires_at, expires_at);

        let purchase = PurchaseAnswer {
            order_id: OrderId::from_uuid(Uuid::from_u128(2)),
            reservation_id,
            quantity,
        };
        assert_eq!(purchase.reservation_id, reservation.reservation_id);

        let state = QueueState::Waiting {
            position: QueuePosition::new(5).unwrap(),
        };
        assert_ne!(state, QueueState::Admitted);
    }

    #[test]
    fn queue_state_json_has_a_state_field() {
        let waiting = QueueState::Waiting {
            position: QueuePosition::new(5).unwrap(),
        };
        assert_eq!(
            serde_json::to_string(&QueueState::WindowOpen).unwrap(),
            r#"{"state":"window_open"}"#
        );
        assert_eq!(
            serde_json::to_string(&waiting).unwrap(),
            r#"{"state":"waiting","position":5}"#
        );
        assert_eq!(
            serde_json::to_string(&QueueState::Admitted).unwrap(),
            r#"{"state":"admitted"}"#
        );
        for state in [QueueState::WindowOpen, waiting, QueueState::Admitted] {
            let json = serde_json::to_string(&state).unwrap();
            assert_eq!(serde_json::from_str::<QueueState>(&json).unwrap(), state);
        }
    }

    #[test]
    fn json_round_trip_works_and_a_bad_quantity_is_rejected() {
        let reservation = ReservationAnswer {
            reservation_id: ReservationId::from_uuid(Uuid::from_u128(1)),
            quantity: TicketQuantity::new(3).unwrap(),
            expires_at: DateTime::<Utc>::from_timestamp(1300, 0).unwrap(),
        };
        let json = serde_json::to_string(&reservation).unwrap();
        assert_eq!(
            serde_json::from_str::<ReservationAnswer>(&json).unwrap(),
            reservation
        );

        // The check of TicketQuantity runs inside the answer too.
        let bad = json.replace("\"quantity\":3", "\"quantity\":99");
        assert_ne!(bad, json, "the replace must really change the text");
        assert!(serde_json::from_str::<ReservationAnswer>(&bad).is_err());
    }
}
