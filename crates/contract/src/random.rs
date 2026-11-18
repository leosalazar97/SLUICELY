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

//! The `RandomSource` trait: where all code gets random numbers.
//!
//! Only the trait lives here. A seeded generator (same seed, same numbers)
//! is built in area A. Policy code never calls a random function directly.

// LEARN: `&mut self` means the source has state that changes on every call
// (like a counter inside). Two sources made from the same seed must give the
// same numbers in the same order. That is how we replay an event for audit
// and how the simulator gives the same result for the same seed.
//
// LEARN: `Send` only (not `Sync`). One owner uses the source at a time, so
// it never needs to be shared between threads at the same time.
/// A source of random numbers that can be replayed.
pub trait RandomSource: Send {
    /// Returns the next random 64-bit number.
    ///
    /// Errors: none.
    /// Guarantees: a source created from the same seed gives the same
    /// sequence every time.
    fn next_u64(&mut self) -> u64;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fake that counts 1, 2, 3... Only for this test.
    struct CountingSource {
        last: u64,
    }

    impl RandomSource for CountingSource {
        fn next_u64(&mut self) -> u64 {
            self.last += 1;
            self.last
        }
    }

    #[test]
    fn random_source_works_as_a_trait_object() {
        let mut source: Box<dyn RandomSource> = Box::new(CountingSource { last: 0 });
        assert_eq!(source.next_u64(), 1);
        assert_eq!(source.next_u64(), 2);
    }
}
