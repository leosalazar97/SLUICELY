## Shared rules for the three AIs

Status: v0.1 · Created by D5/D7. Document 0 wins if there is a conflict.
This file is the communication file between Claude, ChatGPT and Gemini. They do not talk to each other. They read this file.

## 1. How to read this file

1. Read Document 0 and the Technical Document first.
2. Read sections 2 to 6 here (shared rules). Every AI follows them.
3. Read ONLY your own area in section 7. Do not edit code in other areas.

## 2. Language and names

- All code, comments, logs, commits and errors are in simple English.
- Rust: `snake_case` for functions and variables, `PascalCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- TypeScript: `camelCase` for functions and variables, `PascalCase` for types and components.
- Full names only. `reservation`, not `res2`. `event_id`, not `eid`.
- IDs are different types, not plain strings: `EventId`, `SessionId`, `UserId`, `ReservationId`. They are defined once in the contract.
- Time is always UTC. Time and random come in as parameters (a `Clock` and a seeded random source), never read directly in `domain`, `policy`, `controller`.
- Proposed library choices for the contract (confirmed when the contract is frozen): `uuid` for IDs, `chrono` for UTC time, `thiserror` for errors. One-line reason in the commit (Document 0, section 15).

## 3. Errors

- Each module has its own error enum with `thiserror`. Example: `InventoryError`, `PolicyError`.
- Functions return `Result<T, ModuleError>`. No panics in production code.
- Errors are converted at the edge (`api`) into HTTP answers. Inner modules never know about HTTP.
- Error names and variants that cross areas live in the contract.

## 4. Logs

Rules are in Document 0, section 15 (Logs). Short version:

- Use `tracing` macros: `error!`, `warn!`, `info!`, `debug!`.
- Always add fields: `event_id`, `session_id`, `user_id`, `idempotency_key`, `reservation_id`.
- Log message format: a short lowercase sentence, for example `info!(event_id = %event_id, "reservation created")`.
- Every important action logs one line. Every error logs one line at the point where it is handled (not at every layer).
- Never log secrets. Never log in simulator hot loops.

## 5. Comments

Rules are in Document 0, section 15 (Comments). Short version:

- File header: what the file does and what it contains.
- Function comment: what it does, errors it can return, what it guarantees.
- Hard code: `LEARN:` comment in simple words. Only Leonardo removes it.

## 6. Shared file comment block

Put this block at the top of every file where two areas meet, and at the top of every contract file.

```rust
// ============================================================
// SHARED FILE
// Owner area: B (Claude). Others: do not edit. Ask Leonardo.
// Used by: A (policy, controller), C (api)
// Status: CONTRACT: frozen | open
//
// CONNECTION RULES:
// 1. Do not rename or change any public type or function here.
// 2. Add new things only after Leonardo approves a change request.
// 3. Keep this file free of I/O and of real clock or random.
// 4. If you need something else, write a change request.
// ============================================================
```

Change request format (copy into the chat with Leonardo):

```
CHANGE REQUEST
From: <AI name>, area <A|B|C>
File: <path>
What I need: <one or two sentences>
Why: <which task is blocked>
Proposed change: <small code sketch>
Affected areas: <A, B, C>
```

## 7. The three areas

### Area A — Pure core (ChatGPT)

- **Owns:** `domain`, `policy`, `controller`, `sim`.
- **Must not touch:** database, Axum, Tokio tasks of the owner, UI.
- **Uses from the contract:** domain types, `AdmissionPolicy` and `RateController` traits, `Clock` and random source.
- **Delivers:** `RandomWindow` policy, `FixedTokenBucket` baseline, `Adaptive` (AIMD) controller, discrete-event simulator for Scenario A and C.
- **Special rules:** no I/O, no real clock, no direct random. Same seed gives same result. Logs allowed (D6), none in hot loops.
- **Tests:** unit tests plus `proptest` for policy and controller. Tests for these critical modules are written by another AI.
- **Reviewed by:** Claude (area B's AI).

### Area B — Data and concurrency (Claude)

- **Owns:** `store`, `inventory`, `idempotency`, `event_owner`, `payment_mock`, and the migrations.
- **Must not touch:** HTTP routes, UI, simulator internals.
- **Uses from the contract:** domain types, error types, `AdmissionPolicy` and `RateController` traits (injected, never hard-coded).
- **Delivers:** Postgres schema and queries, reservation with expiry, purchase, idempotency storage, one-writer-per-event Tokio task, mock payment.
- **Special rules:** write to Postgres BEFORE answering. Effect and idempotency row in the same transaction. Database constraints are the last defence for I1 and I2. Expiry uses `expires_at`, not memory timers.
- **Tests:** `proptest` for I1-I4 on real Postgres, restart test for I3. Tests for these critical modules are written by another AI.
- **Reviewed by:** Gemini (area C's AI).

### Area C — Edges (Gemini)

- **Owns:** `api`, `metrics`, Next.js UI, Jest, Playwright, k6, Docker Compose, GitHub Actions CI.
- **Must not touch:** the logic inside `domain`, `policy`, `controller`, `inventory`, `idempotency`, `event_owner`.
- **Uses from the contract:** command and answer types of the `event_owner`, error types, the SSE message types.
- **Delivers:** Axum routes, SSE, rate limit, session deduplication, metrics endpoint, minimal UI for queue and purchase, E2E and load tests, CI pipeline.
- **Special rules:** convert module errors into HTTP answers in one place. TypeScript strict, no `any`, one logger helper.
- **Tests:** Jest and React Testing Library, Playwright happy path, k6 scripts. Critical code in `api` that touches admission is reviewed by another AI.
- **Reviewed by:** ChatGPT (area A's AI).

## 8. What every AI must deliver for each task

1. Full files, each with its path on the first line.
2. Comments and logs as required.
3. Tests for the code (or a note saying which AI will write them, for critical code).
4. A short explanation of the critical parts, so Leonardo can explain every line.
5. A list of assumptions and open questions. If unsure, say so. Never invent numbers or benchmarks.

## 9. What every AI must NOT do

- Add features, tools or libraries that are not in Document 0 without asking as a question first.
- Change the contract, other areas, or Document 0.
- Put AI in the critical path (admission, order, inventory).
- Use paid services.
- Skip comments, logs, or tests to make the answer shorter.

## 10. Review checklist (for the reviewing AI)

Check in this order:

1. Are I1, I2, I3 or I4 possibly broken?
2. Any race condition, or any code that bypasses the one-writer rule?
3. Is there a missing test, especially a property test?
4. Does the code follow the contract and this file (names, errors, logs, comments)?
5. Any `unwrap` or `expect` in critical code? Any real clock or random in pure modules?

Review answer format: `BLOCKER` (must fix), `WARNING` (should fix), `NOTE` (optional). Say clearly what is wrong, not only what is good.