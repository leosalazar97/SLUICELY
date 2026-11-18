// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, controller, sim), B (inventory, event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The `Clock` trait: where all code gets the current time.
//!
//! Only the trait lives here. The real clock lives at the edge. Fake clocks
//! for tests and for the simulator live in area A.

use chrono::{DateTime, Utc};

// LEARN: A trait is like a Java interface. `domain`, `policy` and
// `controller` must never read the real time. They receive something that
// implements `Clock`. In production it gives the real time. In tests and in
// the simulator it gives time that we control. Java has the same idea:
// `java.time.Clock` can be injected.
//
// LEARN: `Send + Sync` means the value is safe to share between threads.
// The event owner runs in a Tokio task, and tasks can move between threads.
// Without this, `Arc<dyn Clock>` would not compile in that code.
/// A source of the current time, always in UTC.
pub trait Clock: Send + Sync {
    /// Returns the current time in UTC.
    ///
    /// Errors: none.
    /// Guarantees: the value is in UTC. A fake clock should never go
    /// backwards. Code that uses a real clock must still keep the invariants
    /// if the system time jumps.
    fn now(&self) -> DateTime<Utc>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake that always gives the same time. Only for this test.
    struct FixedClock(DateTime<Utc>);

    impl Clock for FixedClock {
        fn now(&self) -> DateTime<Utc> {
            self.0
        }
    }

    // Compile-time proof: if `dyn Clock` is not Send + Sync, this fails to build.
    fn assert_send_sync<T: ?Sized + Send + Sync>() {}

    #[test]
    fn clock_works_as_a_trait_object_and_is_thread_safe() {
        assert_send_sync::<dyn Clock>();
        let start = DateTime::<Utc>::from_timestamp(0, 0).unwrap();
        let clock: &dyn Clock = &FixedClock(start);
        assert_eq!(clock.now(), start);
    }
}
