// ============================================================
// SHARED FILE
// Owner: contract (written by Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, controller), B (event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! A place in the queue. Place 1 is the first user to be served.

/// A queue place, counted from 1. Place 0 does not exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QueuePosition(u64);

impl QueuePosition {
    /// Builds a position from a number counted from 1.
    ///
    /// Errors: none. It returns `None` for 0, so the caller must handle it.
    /// Guarantees: a position that is returned is 1 or more.
    pub fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    /// Returns the place number, counted from 1.
    ///
    /// Errors: none.
    /// Guarantees: the value is 1 or more.
    pub fn get(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for QueuePosition {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_not_a_position() {
        assert_eq!(QueuePosition::new(0), None);
    }

    #[test]
    fn keeps_its_number_orders_and_displays() {
        let first = QueuePosition::new(1).unwrap();
        let second = QueuePosition::new(2).unwrap();
        assert_eq!(first.get(), 1);
        assert!(first < second);
        assert_eq!(second.to_string(), "2");
    }
}
