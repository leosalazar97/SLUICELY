// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: A (controller, sim), B (event_owner), C (api, metrics)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The `RateController` trait: the SPEED of admission.
//!
//! The controller reads how the protected backend is doing and gives a rate.
//! It can never change the order of users and can never touch inventory
//! (Technical Document, section 7).

use chrono::{DateTime, Utc};
use std::time::Duration;
use thiserror::Error;

/// How many users may enter per second. The value 0 means "admit nobody".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AdmissionRate(u32);

impl AdmissionRate {
    /// Builds a rate from users per second.
    ///
    /// Errors: none.
    /// Guarantees: `users_per_second` returns the same number.
    pub const fn from_users_per_second(value: u32) -> Self {
        Self(value)
    }

    /// Returns the rate in users per second.
    ///
    /// Errors: none.
    /// Guarantees: returns exactly the number given to `from_users_per_second`.
    pub const fn users_per_second(&self) -> u32 {
        self.0
    }
}

// LEARN: `std::time::Duration` is a length of time (for example 20 ms). It is
// not a point in time. It is like Java's `java.time.Duration`. The moment of
// the measurement is a separate field, `observed_at`, and it is given to us:
// the controller never reads the clock itself.
/// What the protected backend did during one measurement period.
///
/// Counts are used instead of an error rate in float form. So the controller
/// has no rounding or NaN problem, and the same input gives the same output.
///
/// Which latency statistic goes in `latency` (for example p95) is not
/// decided here. The controller ADR decides it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendObservation {
    observed_at: DateTime<Utc>,
    latency: Duration,
    request_count: u32,
    error_count: u32,
}

/// Why an observation was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ObservationError {
    /// There are more errors than requests.
    #[error("error count {error_count} is bigger than request count {request_count}")]
    MoreErrorsThanRequests {
        error_count: u32,
        request_count: u32,
    },
}

impl BackendObservation {
    /// Checks the counts and builds an observation.
    ///
    /// Errors: `MoreErrorsThanRequests` if `error_count > request_count`.
    /// Guarantees: a returned observation has `error_count <= request_count`.
    /// Zero requests and zero errors is allowed (a quiet period).
    pub fn new(
        observed_at: DateTime<Utc>,
        latency: Duration,
        request_count: u32,
        error_count: u32,
    ) -> Result<Self, ObservationError> {
        if error_count > request_count {
            return Err(ObservationError::MoreErrorsThanRequests {
                error_count,
                request_count,
            });
        }
        Ok(Self {
            observed_at,
            latency,
            request_count,
            error_count,
        })
    }

    /// When the period was measured (UTC). Errors: none.
    pub fn observed_at(&self) -> DateTime<Utc> {
        self.observed_at
    }

    /// The latency summary of the period. Errors: none.
    pub fn latency(&self) -> Duration {
        self.latency
    }

    /// How many requests the backend got in the period. Errors: none.
    pub fn request_count(&self) -> u32 {
        self.request_count
    }

    /// How many of those requests failed. Never above `request_count`. Errors: none.
    pub fn error_count(&self) -> u32 {
        self.error_count
    }
}

// LEARN: `&mut self` in `observe` means the controller keeps a state that
// changes (for example the current rate). `Send` is needed because the owner
// task holds the controller, and tasks can move between threads. There is no
// `Sync`, because one owner uses the controller at a time.
/// Decides how fast users are admitted.
///
/// Every implementation must keep these rules:
/// 1. It only decides speed. It never changes order and never touches inventory.
/// 2. It is pure: no real clock, no random, no I/O. Time arrives inside the
///    observation.
/// 3. The same sequence of observations always gives the same sequence of rates.
///
/// The baseline (`FixedTokenBucket`) ignores the observations and always gives
/// the same rate. The adaptive controller changes the rate.
pub trait RateController: Send {
    /// Returns the rate to use now.
    ///
    /// Errors: none.
    /// Guarantees: it works before the first observation (the starting rate).
    fn current_rate(&self) -> AdmissionRate;

    /// Gives the controller one new measurement.
    ///
    /// Errors: none.
    /// Guarantees: the next `current_rate` may be different. Observations
    /// arrive in time order.
    fn observe(&mut self, observation: &BackendObservation);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(seconds, 0).unwrap()
    }

    /// A fake controller with a fixed rate. Only for this test.
    struct FixedRate(AdmissionRate);

    impl RateController for FixedRate {
        fn current_rate(&self) -> AdmissionRate {
            self.0
        }

        fn observe(&mut self, _observation: &BackendObservation) {}
    }

    // Compile-time proof: if `dyn RateController` is not Send, this fails to build.
    fn assert_send<T: ?Sized + Send>() {}

    #[test]
    fn rate_keeps_its_value_and_zero_is_allowed() {
        assert_eq!(
            AdmissionRate::from_users_per_second(50).users_per_second(),
            50
        );
        assert_eq!(
            AdmissionRate::from_users_per_second(0).users_per_second(),
            0
        );
        assert!(AdmissionRate::from_users_per_second(1) < AdmissionRate::from_users_per_second(2));
    }

    #[test]
    fn observation_accepts_zero_requests_and_equal_counts() {
        assert!(BackendObservation::new(time(0), Duration::from_millis(20), 0, 0).is_ok());
        assert!(BackendObservation::new(time(0), Duration::from_millis(20), 5, 5).is_ok());
    }

    #[test]
    fn observation_rejects_more_errors_than_requests() {
        assert_eq!(
            BackendObservation::new(time(0), Duration::from_millis(20), 5, 6),
            Err(ObservationError::MoreErrorsThanRequests {
                error_count: 6,
                request_count: 5
            })
        );
    }

    #[test]
    fn controller_works_as_a_trait_object_and_is_send() {
        assert_send::<dyn RateController>();
        let mut controller: Box<dyn RateController> =
            Box::new(FixedRate(AdmissionRate::from_users_per_second(50)));
        let observation =
            BackendObservation::new(time(0), Duration::from_millis(20), 10, 0).unwrap();
        controller.observe(&observation);
        assert_eq!(controller.current_rate().users_per_second(), 50);
    }
}
