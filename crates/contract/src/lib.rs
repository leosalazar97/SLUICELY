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
//! It holds the types, traits and errors that all areas share.
//! This file only declares the modules and re-exports their public items.

pub mod clock;
pub mod ids;
pub mod random;

pub use clock::Clock;
pub use ids::{EventId, ReservationId, SessionId, UserId};
pub use random::RandomSource;
