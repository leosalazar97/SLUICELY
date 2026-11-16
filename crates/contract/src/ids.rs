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

//! Typed IDs.
//!
//! Every kind of ID is its own type. The compiler stops you from passing a
//! `SessionId` where an `EventId` is expected.
//!
//! No ID is created here. There is no random in this file. The edges
//! (api, store, simulator) create new IDs and pass them in.

// LEARN: `macro_rules!` writes code for you at compile time. Think of it as
// a copy-and-paste tool that the compiler understands. Here it makes four
// almost equal ID types, so we do not write the same code four times.
// In Java you would use a code generator or Lombok. `$name` is the slot where
// each ID name goes.
//
// LEARN: `struct EventId(Uuid)` is a "newtype": a struct with one field.
// At runtime it is just a UUID. At compile time `EventId` and `SessionId`
// are different types, so mixing them is a compile error. In Java, passing
// the wrong `UUID` to a method is NOT caught. Here it is.
//
// LEARN: The field is private. The only doors are `from_uuid` and `as_uuid`.
// The store crate uses them to talk to Postgres, so this crate needs no SQLx.
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(::uuid::Uuid);

        impl $name {
            /// Wraps an existing UUID.
            ///
            /// Errors: none.
            /// Guarantees: `as_uuid` returns the same UUID.
            pub const fn from_uuid(value: ::uuid::Uuid) -> Self {
                Self(value)
            }

            /// Returns the UUID inside this ID.
            ///
            /// Errors: none.
            /// Guarantees: returns exactly the UUID given to `from_uuid`.
            pub const fn as_uuid(&self) -> ::uuid::Uuid {
                self.0
            }
        }

        // Display shows the plain UUID. Logs use it: `info!(event_id = %event_id, ...)`.
        impl ::std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&self.0, formatter)
            }
        }
    };
}

define_id! {
    /// Identifies one sales event (for example one concert presale).
    EventId
}

define_id! {
    /// Identifies one browser session (the session cookie).
    SessionId
}

define_id! {
    /// Identifies one user. In v1.0 one session is one user.
    UserId
}

define_id! {
    /// Identifies one reservation of inventory.
    ReservationId
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn id_keeps_its_uuid() {
        let raw = Uuid::from_u128(42);
        assert_eq!(EventId::from_uuid(raw).as_uuid(), raw);
    }

    #[test]
    fn display_shows_the_plain_uuid() {
        let raw = Uuid::from_u128(42);
        assert_eq!(SessionId::from_uuid(raw).to_string(), raw.to_string());
    }

    #[test]
    fn different_values_are_not_equal_and_have_an_order() {
        let first = UserId::from_uuid(Uuid::from_u128(1));
        let second = UserId::from_uuid(Uuid::from_u128(2));
        assert_ne!(first, second);
        assert!(first < second);
    }
}
