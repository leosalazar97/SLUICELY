// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: B (store, event_owner), C (api)
// Status: CONTRACT: open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================

//! The life of a queue entry.
//!
//! `Waiting` becomes `Admitted`, and `Admitted` or `Waiting` becomes `Left`.
//! `Left` is final. The text form (`as_str`, `parse`) is what the database
//! stores, so nobody writes their own.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The state of one queue entry.
// The JSON text must stay equal to `as_str`. A test below checks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueueEntryStatus {
    /// The user waits for admission and has a queue position.
    Waiting,
    /// The user may reserve tickets. In v1.0 an admission does not expire.
    Admitted,
    /// The user left the queue. This is final: the same session cannot join again.
    Left,
}

/// The text is not a known queue entry status.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("unknown queue entry status: {value}")]
pub struct QueueEntryStatusParseError {
    value: String,
}

impl QueueEntryStatus {
    /// Every status, in life order.
    pub const ALL: [QueueEntryStatus; 3] = [
        QueueEntryStatus::Waiting,
        QueueEntryStatus::Admitted,
        QueueEntryStatus::Left,
    ];

    /// Returns the text stored in the database.
    ///
    /// Errors: none.
    /// Guarantees: `parse(status.as_str())` gives back `status`.
    pub fn as_str(&self) -> &'static str {
        match self {
            QueueEntryStatus::Waiting => "waiting",
            QueueEntryStatus::Admitted => "admitted",
            QueueEntryStatus::Left => "left",
        }
    }

    /// Reads a status from the text stored in the database.
    ///
    /// Errors: `QueueEntryStatusParseError` if the text is not one of the three.
    /// Guarantees: exact match only, no trimming and no changing of case.
    pub fn parse(text: &str) -> Result<Self, QueueEntryStatusParseError> {
        Self::ALL
            .into_iter()
            .find(|status| status.as_str() == text)
            .ok_or_else(|| QueueEntryStatusParseError {
                value: text.to_owned(),
            })
    }

    /// True if the status can never change again.
    ///
    /// Errors: none.
    /// Guarantees: true only for `Left`.
    pub fn is_final(&self) -> bool {
        matches!(self, QueueEntryStatus::Left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trip_works_for_every_status() {
        for status in QueueEntryStatus::ALL {
            assert_eq!(QueueEntryStatus::parse(status.as_str()), Ok(status));
        }
    }

    #[test]
    fn unknown_text_is_rejected_exactly() {
        for bad in ["", "Waiting", " waiting", "gone", "left "] {
            assert!(
                QueueEntryStatus::parse(bad).is_err(),
                "should reject {bad:?}"
            );
        }
    }

    #[test]
    fn only_left_is_final() {
        for status in QueueEntryStatus::ALL {
            assert_eq!(status.is_final(), status == QueueEntryStatus::Left);
        }
    }

    #[test]
    fn json_form_is_the_same_text_as_the_database_text() {
        for status in QueueEntryStatus::ALL {
            let json = serde_json::to_string(&status).unwrap();
            assert_eq!(json, format!("\"{}\"", status.as_str()));
            assert_eq!(
                serde_json::from_str::<QueueEntryStatus>(&json).unwrap(),
                status
            );
        }
        assert!(serde_json::from_str::<QueueEntryStatus>("\"Waiting\"").is_err());
    }
}
