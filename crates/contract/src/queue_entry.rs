// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, sim), B (event_owner, store)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! One user who waits in the queue of an event.
//!
//! The entry has no position. The policy only orders entries, and the
//! `event_owner` gives each one a `QueuePosition` when it appends the batch.

use crate::ids::{SessionId, UserId};
use chrono::{DateTime, Utc};

/// A waiting user: who they are and when they joined.
///
/// In v1.0 one session is one user, so each session has exactly one user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueEntry {
    session_id: SessionId,
    user_id: UserId,
    joined_at: DateTime<Utc>,
}

impl QueueEntry {
    /// Builds an entry.
    ///
    /// Errors: none.
    /// Guarantees: the getters return exactly the given values.
    pub fn new(session_id: SessionId, user_id: UserId, joined_at: DateTime<Utc>) -> Self {
        Self {
            session_id,
            user_id,
            joined_at,
        }
    }

    /// The session that joined. Errors: none.
    pub fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// The user who joined. Errors: none.
    pub fn user_id(&self) -> UserId {
        self.user_id
    }

    /// When the user joined (UTC). Errors: none.
    pub fn joined_at(&self) -> DateTime<Utc> {
        self.joined_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn keeps_every_field() {
        let session_id = SessionId::from_uuid(Uuid::from_u128(1));
        let user_id = UserId::from_uuid(Uuid::from_u128(2));
        let joined_at = DateTime::<Utc>::from_timestamp(1000, 0).unwrap();
        let entry = QueueEntry::new(session_id, user_id, joined_at);
        assert_eq!(entry.session_id(), session_id);
        assert_eq!(entry.user_id(), user_id);
        assert_eq!(entry.joined_at(), joined_at);
    }
}
