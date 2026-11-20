// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: B (event_owner), C (api, UI)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The messages that the server pushes to the browser over SSE.
//!
//! The api writes each message as an SSE event: the name from `event_name`
//! and the JSON form as the data. The browser listens with
//! `EventSource.addEventListener(name, ...)`.

use crate::ids::ReservationId;
use crate::owner_answer::QueueState;
use crate::queue_position::QueuePosition;
use serde::{Deserialize, Serialize};

// LEARN: `tag = "type"` writes the variant name in a field "type", for
// example {"type":"waiting","position":5}. It is the same idea as `tag` in
// `OwnerError` and `QueueState`. The JSON text is part of the contract with
// area C: a rename here breaks the browser code.
/// One message pushed to the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseMessage {
    /// The user waits at this place.
    Waiting { position: QueuePosition },
    /// The user is admitted and may reserve.
    Admitted,
    /// The reservation expired and its tickets went back to stock.
    ReservationExpired { reservation_id: ReservationId },
    /// The user joined while the window is open. The order is decided when
    /// the window closes, so there is no position yet.
    WindowOpen,
}

impl SseMessage {
    /// Returns the SSE event name of this message.
    ///
    /// Errors: none.
    /// Guarantees: the name is equal to the `type` field in the JSON form.
    pub fn event_name(&self) -> &'static str {
        match self {
            SseMessage::WindowOpen => "window_open",
            SseMessage::Waiting { .. } => "waiting",
            SseMessage::Admitted => "admitted",
            SseMessage::ReservationExpired { .. } => "reservation_expired",
        }
    }
}

impl From<QueueState> for SseMessage {
    /// Turns the queue state of a `Join` answer into the SSE message.
    fn from(state: QueueState) -> Self {
        match state {
            QueueState::WindowOpen => SseMessage::WindowOpen,
            QueueState::Waiting { position } => SseMessage::Waiting { position },
            QueueState::Admitted => SseMessage::Admitted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn every_message() -> Vec<SseMessage> {
        vec![
            SseMessage::Waiting {
                position: QueuePosition::new(5).unwrap(),
            },
            SseMessage::Admitted,
            SseMessage::ReservationExpired {
                reservation_id: ReservationId::from_uuid(Uuid::from_u128(1)),
            },
            SseMessage::WindowOpen,
        ]
    }

    #[test]
    fn json_form_is_exact() {
        let messages = every_message();
        assert_eq!(
            serde_json::to_string(&messages[0]).unwrap(),
            r#"{"type":"waiting","position":5}"#
        );
        assert_eq!(
            serde_json::to_string(&messages[1]).unwrap(),
            r#"{"type":"admitted"}"#
        );
        assert_eq!(
            serde_json::to_string(&messages[2]).unwrap(),
            r#"{"type":"reservation_expired","reservation_id":"00000000-0000-0000-0000-000000000001"}"#
        );
        assert_eq!(
            serde_json::to_string(&messages[3]).unwrap(),
            r#"{"type":"window_open"}"#
        );
    }

    #[test]
    fn event_name_is_the_type_field_and_the_message_round_trips() {
        for message in every_message() {
            let json = serde_json::to_string(&message).unwrap();
            let value: serde_json::Value = serde_json::from_str(&json).unwrap();
            assert_eq!(value["type"], message.event_name());
            assert_eq!(serde_json::from_str::<SseMessage>(&json).unwrap(), message);
        }
    }

    #[test]
    fn a_queue_state_becomes_the_matching_message() {
        let position = QueuePosition::new(3).unwrap();
        assert_eq!(
            SseMessage::from(QueueState::Waiting { position }),
            SseMessage::Waiting { position }
        );
        assert_eq!(SseMessage::from(QueueState::Admitted), SseMessage::Admitted);
    }
}
