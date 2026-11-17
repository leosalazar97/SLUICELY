// ============================================================
// SHARED FILE
// Owner: contract (written by Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, controller, sim), B (store, event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The settings of one sales event.
//!
//! These are values, not constants in the code: each event has its own
//! window, stock and reservation time. The values are checked once, when the
//! settings are created.

use crate::seed::Seed;
use chrono::{DateTime, Utc};
use thiserror::Error;

/// How long a reservation is held, if the event does not say otherwise.
/// 5 minutes. Decided by Leonardo. It can change after we measure.
pub const DEFAULT_RESERVATION_TTL_SECONDS: u32 = 300;

/// The settings of one event, already checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSettings {
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    total_stock: u32,
    reservation_ttl_seconds: u32,
    seed: Seed,
}

/// Why the settings were rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EventSettingsError {
    /// The window does not end after it starts.
    #[error("the opening window must end after it starts")]
    WindowNotForward,
    /// There is no stock to sell.
    #[error("total stock must be at least 1")]
    NoStock,
    /// A reservation would expire at once.
    #[error("reservation time must be at least 1 second")]
    NoReservationTime,
}

impl EventSettings {
    /// Checks the values and builds the settings.
    ///
    /// Errors: `WindowNotForward`, `NoStock`, `NoReservationTime`. The checks
    /// run in this order.
    /// Guarantees: `window_end` is after `window_start`, `total_stock` is 1 or
    /// more, and `reservation_ttl_seconds` is 1 or more.
    pub fn new(
        window_start: DateTime<Utc>,
        window_end: DateTime<Utc>,
        total_stock: u32,
        reservation_ttl_seconds: u32,
        seed: Seed,
    ) -> Result<Self, EventSettingsError> {
        if window_end <= window_start {
            return Err(EventSettingsError::WindowNotForward);
        }
        if total_stock == 0 {
            return Err(EventSettingsError::NoStock);
        }
        if reservation_ttl_seconds == 0 {
            return Err(EventSettingsError::NoReservationTime);
        }
        Ok(Self {
            window_start,
            window_end,
            total_stock,
            reservation_ttl_seconds,
            seed,
        })
    }

    /// When the opening window starts (UTC). Errors: none.
    pub fn window_start(&self) -> DateTime<Utc> {
        self.window_start
    }

    /// When the opening window ends (UTC). Always after `window_start`. Errors: none.
    pub fn window_end(&self) -> DateTime<Utc> {
        self.window_end
    }

    /// How many tickets exist in total. Always 1 or more. Errors: none.
    pub fn total_stock(&self) -> u32 {
        self.total_stock
    }

    /// How many seconds a reservation is held. Always 1 or more. Errors: none.
    pub fn reservation_ttl_seconds(&self) -> u32 {
        self.reservation_ttl_seconds
    }

    /// The random seed saved with the event. Errors: none.
    pub fn seed(&self) -> Seed {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(seconds, 0).unwrap()
    }

    #[test]
    fn accepts_good_settings_and_keeps_every_value() {
        let settings =
            EventSettings::new(time(1000), time(2000), 500, 300, Seed::from_u64(9)).unwrap();
        assert_eq!(settings.window_start(), time(1000));
        assert_eq!(settings.window_end(), time(2000));
        assert_eq!(settings.total_stock(), 500);
        assert_eq!(settings.reservation_ttl_seconds(), 300);
        assert_eq!(settings.seed(), Seed::from_u64(9));
    }

    #[test]
    fn rejects_a_window_that_is_empty_or_backwards() {
        for (start, end) in [(1000, 1000), (2000, 1000)] {
            assert_eq!(
                EventSettings::new(time(start), time(end), 10, 300, Seed::from_u64(1)),
                Err(EventSettingsError::WindowNotForward)
            );
        }
    }

    #[test]
    fn rejects_zero_stock() {
        assert_eq!(
            EventSettings::new(time(1000), time(2000), 0, 300, Seed::from_u64(1)),
            Err(EventSettingsError::NoStock)
        );
    }

    #[test]
    fn rejects_zero_reservation_time() {
        assert_eq!(
            EventSettings::new(time(1000), time(2000), 10, 0, Seed::from_u64(1)),
            Err(EventSettingsError::NoReservationTime)
        );
    }

    #[test]
    fn default_reservation_time_is_five_minutes() {
        assert_eq!(DEFAULT_RESERVATION_TTL_SECONDS, 5 * 60);
    }
}
