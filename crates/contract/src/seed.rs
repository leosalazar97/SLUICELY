// ============================================================
// SHARED FILE
// Owner: contract (written by Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, sim), B (store, event_owner)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The random seed of an event.
//!
//! The seed is saved with the event. With the same seed, the policy gives the
//! same order, so a result can be replayed and audited (Technical Document,
//! section 6).

/// A random seed. The same seed must give the same sequence of numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    /// Creates a seed from a number.
    ///
    /// Errors: none.
    /// Guarantees: `as_u64` returns the same number.
    pub const fn from_u64(value: u64) -> Self {
        Self(value)
    }

    /// Returns the seed as a number.
    ///
    /// Errors: none.
    /// Guarantees: returns exactly the number given to `from_u64`.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    // LEARN: Postgres has no unsigned 64-bit type. Its `BIGINT` is a signed
    // 64-bit number (`i64`). `as i64` keeps the same 64 bits and only changes
    // how we read them: big values become negative numbers. Nothing is lost
    // and nothing panics. Java's `long` is signed too, so this is like storing
    // a Java `long`. The store crate uses these two functions, so nobody
    // writes their own conversion.
    /// Returns the same 64 bits as a signed number, for a Postgres `BIGINT`.
    ///
    /// Errors: none.
    /// Guarantees: `from_i64_bits(seed.to_i64_bits())` gives back `seed`.
    pub const fn to_i64_bits(&self) -> i64 {
        self.0 as i64
    }

    /// Builds a seed from the signed number read from a Postgres `BIGINT`.
    ///
    /// Errors: none.
    /// Guarantees: the reverse of `to_i64_bits`, for every possible value.
    pub const fn from_i64_bits(value: i64) -> Self {
        Self(value as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_keeps_its_number() {
        assert_eq!(Seed::from_u64(7).as_u64(), 7);
    }

    #[test]
    fn database_conversion_loses_nothing() {
        // Edge values: zero, the first value that becomes negative, the maximum.
        for value in [0, 1, i64::MAX as u64, 1 << 63, u64::MAX] {
            let seed = Seed::from_u64(value);
            assert_eq!(Seed::from_i64_bits(seed.to_i64_bits()), seed);
        }
    }

    #[test]
    fn big_seed_becomes_a_negative_database_number() {
        // Proves that we really reinterpret the bits and do not clamp the value.
        assert_eq!(Seed::from_u64(u64::MAX).to_i64_bits(), -1);
    }
}
