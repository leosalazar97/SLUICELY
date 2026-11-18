// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, controller), B (store, event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

#![deny(clippy::unwrap_used, clippy::expect_used)]

//! The contract crate.
//!
//! It holds the types, traits and errors that all areas share.
//! This file only declares the modules and re-exports their public items.

pub mod admission_policy;
pub mod clock;
pub mod event_settings;
pub mod idempotency_key;
pub mod ids;
pub mod owner_command;
pub mod owner_error;
pub mod queue_entry;
pub mod queue_entry_status;
pub mod queue_position;
pub mod random;
pub mod rate_controller;
pub mod reservation_status;
pub mod seed;
pub mod ticket_quantity;

pub use admission_policy::{AdmissionPolicy, PolicyError};
pub use clock::Clock;
pub use event_settings::{EventSettings, EventSettingsError, DEFAULT_RESERVATION_TTL_SECONDS};
pub use idempotency_key::{IdempotencyKey, IdempotencyKeyError, MAX_IDEMPOTENCY_KEY_LENGTH};
pub use ids::{EventId, OrderId, ReservationId, SessionId, UserId};
pub use owner_command::OwnerCommand;
pub use owner_error::OwnerError;
pub use queue_entry::QueueEntry;
pub use queue_entry_status::{QueueEntryStatus, QueueEntryStatusParseError};
pub use queue_position::QueuePosition;
pub use random::RandomSource;
pub use rate_controller::{AdmissionRate, BackendObservation, ObservationError, RateController};
pub use reservation_status::{ReservationStatus, ReservationStatusParseError};
pub use seed::Seed;
pub use ticket_quantity::{TicketQuantity, TicketQuantityError, MAX_TICKETS_PER_USER};
