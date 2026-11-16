// ============================================================
// SHARED FILE
// Owner: contract (written by Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, controller), B (store, event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The contract crate.
//!
//! It will hold the types, traits and errors that all areas share.
//! Right now it only has one tiny function, so we can check that the
//! workspace, the lints and the tests all work.

/// Returns the project name.
///
/// Errors: none.
/// Guarantees: always returns the same text.
pub fn project_name() -> &'static str {
    "sluicely"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_name_is_sluicely() {
        assert_eq!(project_name(), "sluicely");
    }
}
