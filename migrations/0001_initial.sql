-- ============================================================
-- SHARED FILE
-- Owner area: B (Claude). Others: do not edit. Ask Leonardo.
-- Used by: B (store, inventory, idempotency, event_owner), C (docker, CI)
-- Status: CONTRACT: open
--
-- CONNECTION RULES:
-- 1. Do not rename or change any table or column here.
-- 2. Add new things only in a NEW migration, after Leonardo approves
--    a change request.
-- 3. Only migrations change the schema (Document 0, section 15).
-- 4. If you need something else, write a change request.
-- ============================================================

-- Migration 0001: the first schema. Part 1 has the events and the queue.
-- Part 2 has the reservations, the orders and the idempotency keys.--
-- The database is the last defence for the invariants (Technical Document,
-- section 5). The owner also checks, but a rule written here cannot be
-- skipped by a bug in the code.
--
-- Contract links (a test in the store crate will check them later):
-- - Statuses are text. The allowed values are the `as_str` values of
--   `QueueEntryStatus` and `ReservationStatus` in the contract crate.
-- - `seed` is a signed 64-bit number. It holds the bits of `Seed`, see
--   `Seed::to_i64_bits`.
-- - Postgres has no unsigned numbers. Stock and TTL are `u32` in the
--   contract, so their columns are `bigint` with an upper CHECK of 4294967295.
-- - `position` is `bigint`. `QueuePosition` stops at `i64::MAX`.
-- - Inventory is counted in tickets, not in rows (D8).
-- - All times are `timestamptz`. All values are in UTC (Document 0).
-- - No table creates IDs. The edges create them (the contract says so).

-- LEARN: A CHECK rule is tested by Postgres on every INSERT and UPDATE. If it
-- is false, the whole statement fails and nothing changes. It is like a
-- Bean Validation rule in Java, but it cannot be bypassed: even a bug in our
-- code, or a person using psql, must follow it.
CREATE TABLE events (
    event_id                uuid        PRIMARY KEY,
    total_inventory         bigint      NOT NULL,
    sold                    bigint      NOT NULL DEFAULT 0,
    reserved                bigint      NOT NULL DEFAULT 0,
    window_start            timestamptz NOT NULL,
    window_end              timestamptz NOT NULL,
    reservation_ttl_seconds bigint      NOT NULL,
    seed                    bigint      NOT NULL,

    CONSTRAINT events_total_inventory_positive
        CHECK (total_inventory >= 1),
    -- The contract holds stock as `u32`. The database must refuse any number
    -- that Rust cannot read back.
    CONSTRAINT events_total_inventory_fits_u32
        CHECK (total_inventory <= 4294967295),
    CONSTRAINT events_sold_not_negative
        CHECK (sold >= 0),
    CONSTRAINT events_reserved_not_negative
        CHECK (reserved >= 0),
    -- I1: never more tickets sold or held than the total. Both counters are
    -- >= 0, so each one is also <= total <= the u32 maximum. They need no
    -- upper check of their own.
    CONSTRAINT events_inventory_not_oversold
        CHECK (sold + reserved <= total_inventory),
    CONSTRAINT events_window_goes_forward
        CHECK (window_end > window_start),
    CONSTRAINT events_reservation_ttl_positive
        CHECK (reservation_ttl_seconds >= 1),
    CONSTRAINT events_reservation_ttl_fits_u32
        CHECK (reservation_ttl_seconds <= 4294967295)
);

-- One row per session that joined an event.
--
-- `position` is empty while the opening window is open. The order is decided
-- when the window closes (RandomWindow). A user who joins after the window
-- closes gets a position at once, at the end of the queue.
CREATE TABLE queue_entries (
    event_id  uuid        NOT NULL REFERENCES events (event_id),
    session_id uuid       NOT NULL,
    user_id   uuid        NOT NULL,
    status    text        NOT NULL,
    position  bigint,
    joined_at timestamptz NOT NULL,

    -- I2: one entry per session in an event. The Technical Document calls
    -- this UNIQUE (event_id, session_id). A primary key is the same rule.
    PRIMARY KEY (event_id, session_id),

    -- I2: one entry per user in an event. In v1.0 one session is one user,
    -- so this is a second defence.
    CONSTRAINT queue_entries_user_once_per_event
        UNIQUE (event_id, user_id),

    CONSTRAINT queue_entries_status_known
        CHECK (status IN ('waiting', 'admitted', 'left')),
    CONSTRAINT queue_entries_position_positive
        CHECK (position IS NULL OR position >= 1),
    -- Admission follows the order, so an admitted entry always has a place.
    CONSTRAINT queue_entries_admitted_has_position
        CHECK (status <> 'admitted' OR position IS NOT NULL)
);

-- LEARN: A partial unique index applies the rule only to the rows that match
-- its WHERE part. Here, two entries of one event cannot have the same
-- position, but many entries can have no position yet (NULL), because
-- the window is still open. Postgres treats each NULL as different, and the
-- WHERE part makes this clear for the reader.
CREATE UNIQUE INDEX queue_entries_position_unique
    ON queue_entries (event_id, position)
    WHERE position IS NOT NULL;
    
-- ------------------------------------------------------------
-- Part 2: reservations, orders and idempotency keys.
-- ------------------------------------------------------------

-- One row per reservation. A reservation holds tickets from the stock.
--
-- Contract links: `status` uses the `as_str` values of `ReservationStatus`.
-- The quantity range is `TicketQuantity` (1 to MAX_TICKETS_PER_USER = 4).
CREATE TABLE reservations (
    reservation_id uuid        PRIMARY KEY,
    event_id       uuid        NOT NULL REFERENCES events (event_id),
    user_id        uuid        NOT NULL,
    quantity       integer     NOT NULL,
    status         text        NOT NULL,
    created_at     timestamptz NOT NULL,
    expires_at     timestamptz NOT NULL,

    CONSTRAINT reservations_quantity_in_range
        CHECK (quantity BETWEEN 1 AND 4),
    CONSTRAINT reservations_status_known
        CHECK (status IN ('reserved', 'purchased', 'expired', 'cancelled')),
    CONSTRAINT reservations_expires_after_creation
        CHECK (expires_at > created_at),

    -- LEARN: This looks useless, because `reservation_id` is already unique.
    -- It exists so that `orders` can point to ALL FOUR columns at once. A
    -- foreign key needs a unique rule on the columns it points to.
    CONSTRAINT reservations_key_for_orders
        UNIQUE (reservation_id, event_id, user_id, quantity)
);

-- I2: a user holds at most one active or bought reservation per event.
-- `expired` and `cancelled` do not hold stock, so they are not counted. The
-- statuses here are the ones where `ReservationStatus::holds_stock` is true.
CREATE UNIQUE INDEX reservations_one_holding_per_user
    ON reservations (event_id, user_id)
    WHERE status IN ('reserved', 'purchased');

-- The owner looks for reservations to expire. Only `reserved` rows can expire.
CREATE INDEX reservations_due_to_expire
    ON reservations (event_id, expires_at)
    WHERE status = 'reserved';

-- One row per completed purchase.
CREATE TABLE orders (
    order_id       uuid        PRIMARY KEY,
    event_id       uuid        NOT NULL,
    user_id        uuid        NOT NULL,
    reservation_id uuid        NOT NULL,
    quantity       integer     NOT NULL,
    created_at     timestamptz NOT NULL,

    CONSTRAINT orders_quantity_in_range
        CHECK (quantity BETWEEN 1 AND 4),
    -- I2: a reservation is bought once.
    CONSTRAINT orders_one_per_reservation
        UNIQUE (reservation_id),
    -- I2: a user buys once per event (the error `AlreadyPurchased`).
    CONSTRAINT orders_one_per_user_and_event
        UNIQUE (event_id, user_id),
    -- The order has the same event, user and quantity as its reservation.
    CONSTRAINT orders_match_their_reservation
        FOREIGN KEY (reservation_id, event_id, user_id, quantity)
        REFERENCES reservations (reservation_id, event_id, user_id, quantity)
);

-- I4: what we answered for each idempotency key.
--
-- Contract links: `request` is the JSON of `IdempotentRequest`. `response` is
-- the JSON of `StoredResponse`. `operation` is `IdempotentOperation::as_str`.
-- The key text rule is `IdempotencyKey::parse`: visible ASCII, 1 to 128.
-- `TemporarilyUnavailable` and `IdempotencyKeyReused` are never stored.
--
-- LEARN: A CHECK rule that gives NULL counts as PASSED. If a JSON object has
-- no "operation" field, `request ->> 'operation'` is NULL, and a plain
-- comparison would be NULL too, so a bad row would get in. That is why the
-- rules below use `IS NOT DISTINCT FROM` and `IS NOT NULL`.
CREATE TABLE idempotency_keys (
    event_id        uuid        NOT NULL REFERENCES events (event_id),
    user_id         uuid        NOT NULL,
    idempotency_key text        NOT NULL,
    operation       text        NOT NULL,
    request         jsonb       NOT NULL,
    response        jsonb       NOT NULL,
    created_at      timestamptz NOT NULL,

    -- One key means one request, in one event, for one user.
    PRIMARY KEY (event_id, user_id, idempotency_key),

    CONSTRAINT idempotency_keys_key_shape
        CHECK (idempotency_key ~ '^[!-~]{1,128}$'),
    CONSTRAINT idempotency_keys_operation_known
        CHECK (operation IN ('reserve', 'buy')),
    CONSTRAINT idempotency_keys_request_matches_row
        CHECK (
            jsonb_typeof(request) = 'object'
            AND (request ->> 'operation') IS NOT DISTINCT FROM operation
            AND (request ->> 'user_id') IS NOT DISTINCT FROM user_id::text
        ),
    -- Mirrors `StoredResponse::fits`: a reservation answer only for Reserve,
    -- a purchase answer only for Buy, a failure for both.
    CONSTRAINT idempotency_keys_response_fits_operation
        CHECK (
            jsonb_typeof(response) = 'object'
            AND (response ->> 'outcome') IS NOT NULL
            AND (
                (operation = 'reserve'
                    AND response ->> 'outcome' IN ('reserved', 'failed'))
                OR (operation = 'buy'
                    AND response ->> 'outcome' IN ('purchased', 'failed'))
            )
        )
);