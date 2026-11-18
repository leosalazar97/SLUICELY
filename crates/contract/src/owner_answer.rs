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
//! fields are public.

use crate::ids::{OrderId, ReservationId};
use crate::queue_position::QueuePosition;
use crate::ticket_quantity::TicketQuantity;
use chrono::{DateTime, Utc};

/// Where a user is in the queue right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueState {
    /// Waiting, at this place.
    Waiting { position: QueuePosition },
    /// Admitted: the user may reserve.
    Admitted,
}

/// A reservation that was created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReservationAnswer {
    pub reservation_id: ReservationId,
    pub quantity: TicketQuantity,
    /// After this time (UTC) the tickets go back to stock.
    pub expires_at: DateTime<Utc>,
}

/// A purchase that was completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}
