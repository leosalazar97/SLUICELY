// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, sim), B (event_owner)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The `AdmissionPolicy` trait: the ORDER in which users are served.
//!
//! The policy only decides order. The speed of admission is decided by the
//! `RateController`, and the policy never touches inventory.

use crate::event_settings::EventSettings;
use crate::ids::SessionId;
use crate::queue_entry::QueueEntry;
use crate::random::RandomSource;
use thiserror::Error;

/// Why a policy could not order a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PolicyError {
    /// The same session is in the batch twice. The database blocks this, so
    /// it means a bug in the caller.
    #[error("session {session_id} appears more than once in the same batch")]
    DuplicateSession { session_id: SessionId },
}

// LEARN: `&[QueueEntry]` is a "slice": a read-only view of a list. Nothing is
// copied. It is like passing a Java `List<QueueEntry>` that the method may
// read but not change. `&mut dyn RandomSource` means: "any random source, and
// I may change its inner state". `dyn` is the same idea as a Java interface
// type: the real class is chosen by the caller.
//
// LEARN: `Send + Sync` is needed because one policy object is shared by the
// owner task, and tasks can move between threads. The method takes `&self`,
// so a policy keeps no changing state. All changing state (the random
// numbers) comes in through `random`.
/// Orders a batch of waiting users.
///
/// The `event_owner` calls it with the entries it has NOT ordered yet, in the
/// order they were stored. It appends the returned batch after the entries it
/// already ordered. So an earlier position never changes (invariant I3).
///
/// Every implementation must keep these rules:
/// 1. The result has the same entries as the input: none lost, none added.
/// 2. The same settings, the same entries and the same random state give the
///    same result (replay and audit).
/// 3. It only orders. It never reads the real clock, never calls random
///    directly, never does I/O, and never touches inventory.
pub trait AdmissionPolicy: Send + Sync {
    /// Returns the entries in the order they will be served.
    ///
    /// Errors: `PolicyError::DuplicateSession` if a session is twice in the batch.
    /// Guarantees: rules 1 to 3 above. An empty batch gives an empty result.
    fn order(
        &self,
        settings: &EventSettings,
        entries: &[QueueEntry],
        random: &mut dyn RandomSource,
    ) -> Result<Vec<QueueEntry>, PolicyError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::UserId;
    use crate::seed::Seed;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(seconds, 0).unwrap()
    }

    /// A fake policy that keeps the input order. Only for this test.
    struct KeepOrder;

    impl AdmissionPolicy for KeepOrder {
        fn order(
            &self,
            _settings: &EventSettings,
            entries: &[QueueEntry],
            _random: &mut dyn RandomSource,
        ) -> Result<Vec<QueueEntry>, PolicyError> {
            Ok(entries.to_vec())
        }
    }

    /// A fake random source. Only for this test.
    struct NoRandom;

    impl RandomSource for NoRandom {
        fn next_u64(&mut self) -> u64 {
            0
        }
    }

    // Compile-time proof: if `dyn AdmissionPolicy` is not Send + Sync, this fails to build.
    fn assert_send_sync<T: ?Sized + Send + Sync>() {}

    #[test]
    fn policy_works_as_a_thread_safe_trait_object() {
        assert_send_sync::<dyn AdmissionPolicy>();
        let settings =
            EventSettings::new(time(1000), time(2000), 10, 300, Seed::from_u64(1)).unwrap();
        let entries = vec![
            QueueEntry::new(
                SessionId::from_uuid(Uuid::from_u128(1)),
                UserId::from_uuid(Uuid::from_u128(1)),
                time(1100),
            ),
            QueueEntry::new(
                SessionId::from_uuid(Uuid::from_u128(2)),
                UserId::from_uuid(Uuid::from_u128(2)),
                time(1200),
            ),
        ];
        let policy: &dyn AdmissionPolicy = &KeepOrder;
        let mut random = NoRandom;
        assert_eq!(
            policy.order(&settings, &entries, &mut random),
            Ok(entries.clone())
        );
    }

    #[test]
    fn duplicate_session_error_names_the_session() {
        let session_id = SessionId::from_uuid(Uuid::from_u128(7));
        let error = PolicyError::DuplicateSession { session_id };
        assert_eq!(
            error.to_string(),
            format!("session {session_id} appears more than once in the same batch")
        );
    }
}
