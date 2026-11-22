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

//! The commands that go to the `event_owner`.
//!
//! The owner handles them one by one (Technical Document, section 4). A
//! command has no `event_id` (one owner serves one event) and no time (the
//! owner reads its own `Clock`). The reply channel is not here: area B wraps
//! each command with its own reply sender.

use crate::idempotency_key::IdempotencyKey;
use crate::idempotency_record::IdempotentRequest;
use crate::ids::{ReservationId, SessionId, UserId};
use crate::ticket_quantity::TicketQuantity;

/// One request to the owner of an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerCommand {
    /// A session joins the queue. In v1.0 one session is one user.
    Join {
        session_id: SessionId,
        user_id: UserId,
    },
    /// A session leaves the queue.
    Leave { session_id: SessionId },
    /// Read only. Asks where a session is now: its queue state, its active
    /// reservation and its order. Changes nothing, so it has no key. The page
    /// uses it after a reload or a reconnect.
    Status { session_id: SessionId },
    /// An admitted user reserves tickets. Carries an idempotency key (I4).
    Reserve {
        user_id: UserId,
        quantity: TicketQuantity,
        idempotency_key: IdempotencyKey,
    },
    /// A user buys a reservation. Carries an idempotency key (I4).
    Buy {
        user_id: UserId,
        reservation_id: ReservationId,
        idempotency_key: IdempotencyKey,
    },
    /// Internal: the owner sends it to itself on a tick. Admits users at the
    /// current `AdmissionRate`. The api never sends it.
    AdmitDue,
    /// Internal: the owner sends it to itself on a tick. Expires reservations
    /// whose `expires_at` has passed. The api never sends it.
    ExpireDue,
}

impl OwnerCommand {
    // LEARN: This `match` lists every variant on purpose. It has no `_`
    // catch-all. If someone adds a new command later, the compiler stops here
    // and forces a choice: does the new command carry a key or not? A `_`
    // arm would hide the question, and a missing key means a broken I4.
    /// Returns the idempotency key of the command, if it has one.
    ///
    /// Errors: none.
    /// Guarantees: `Some` for `Reserve` and `Buy`, `None` for all others.
    pub fn idempotency_key(&self) -> Option<&IdempotencyKey> {
        match self {
            OwnerCommand::Reserve {
                idempotency_key, ..
            }
            | OwnerCommand::Buy {
                idempotency_key, ..
            } => Some(idempotency_key),
            OwnerCommand::Join { .. }
            | OwnerCommand::Leave { .. }
            | OwnerCommand::Status { .. }
            | OwnerCommand::AdmitDue
            | OwnerCommand::ExpireDue => None,
        }
    }

    /// Returns the request that the idempotency table compares on a retry.
    ///
    /// Errors: none.
    /// Guarantees: `Some` exactly when `idempotency_key` is `Some`. The
    /// request has the same user and the same quantity or reservation as
    /// the command. It does not hold the key.
    pub fn idempotent_request(&self) -> Option<IdempotentRequest> {
        match self {
            OwnerCommand::Reserve {
                user_id, quantity, ..
            } => Some(IdempotentRequest::Reserve {
                user_id: *user_id,
                quantity: *quantity,
            }),
            OwnerCommand::Buy {
                user_id,
                reservation_id,
                ..
            } => Some(IdempotentRequest::Buy {
                user_id: *user_id,
                reservation_id: *reservation_id,
            }),
            OwnerCommand::Join { .. }
            | OwnerCommand::Leave { .. }
            | OwnerCommand::Status { .. }
            | OwnerCommand::AdmitDue
            | OwnerCommand::ExpireDue => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn only_reserve_and_buy_carry_a_key() {
        let key = IdempotencyKey::parse("key-1").unwrap();
        let session_id = SessionId::from_uuid(Uuid::from_u128(1));
        let user_id = UserId::from_uuid(Uuid::from_u128(2));
        let quantity = TicketQuantity::new(2).unwrap();
        let reservation_id = ReservationId::from_uuid(Uuid::from_u128(3));

        let with_key = [
            OwnerCommand::Reserve {
                user_id,
                quantity,
                idempotency_key: key.clone(),
            },
            OwnerCommand::Buy {
                user_id,
                reservation_id,
                idempotency_key: key.clone(),
            },
        ];
        for command in &with_key {
            assert_eq!(command.idempotency_key(), Some(&key));
        }

        let without_key = [
            OwnerCommand::Join {
                session_id,
                user_id,
            },
            OwnerCommand::Leave { session_id },
            OwnerCommand::Status { session_id },
            OwnerCommand::AdmitDue,
            OwnerCommand::ExpireDue,
        ];
        for command in &without_key {
            assert_eq!(command.idempotency_key(), None);
        }
    }

    #[test]
    fn a_request_exists_exactly_when_a_key_exists() {
        let key = IdempotencyKey::parse("key-1").unwrap();
        let session_id = SessionId::from_uuid(Uuid::from_u128(1));
        let user_id = UserId::from_uuid(Uuid::from_u128(2));
        let quantity = TicketQuantity::new(2).unwrap();
        let reservation_id = ReservationId::from_uuid(Uuid::from_u128(3));

        let commands = [
            OwnerCommand::Reserve {
                user_id,
                quantity,
                idempotency_key: key.clone(),
            },
            OwnerCommand::Buy {
                user_id,
                reservation_id,
                idempotency_key: key,
            },
            OwnerCommand::Join {
                session_id,
                user_id,
            },
            OwnerCommand::Leave { session_id },
            OwnerCommand::Status { session_id },
            OwnerCommand::AdmitDue,
            OwnerCommand::ExpireDue,
        ];
        for command in &commands {
            assert_eq!(
                command.idempotent_request().is_some(),
                command.idempotency_key().is_some()
            );
        }
        assert_eq!(
            commands[0].idempotent_request(),
            Some(IdempotentRequest::Reserve { user_id, quantity })
        );
        assert_eq!(
            commands[1].idempotent_request(),
            Some(IdempotentRequest::Buy {
                user_id,
                reservation_id
            })
        );
    }
}
