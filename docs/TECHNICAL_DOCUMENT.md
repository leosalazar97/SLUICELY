# Technical Document v0: SLUICELY

Status: DRAFT v0.3. Depends on Document 0 (v0.4). If this file and Document 0 disagree, Document 0 wins.
Rule: this file describes the design. ADRs come later, after the discovery work.
Scope: this file describes v1.0 only (see D1 in Document 0, section 16). Multi-event (v1.1), FIFO (v1.2) and Grafana (v1.3) are out of scope here.
Change in v0.3: working method changed by D5 (three AIs, no auto-writing tools). The design itself did not change.

## 1. Checkpoints (rough proposal, to be confirmed)

Budget: about 3-4 hours per week. Target: 31 July 2027 (about 9 months). The checkpoints M0-M4 are internal steps. Together they make v1.0.

| Checkpoint | Goal | Proposed period | Done when |
| --- | --- | --- | --- |
| M0 | Setup, contract and learning | Nov-Dec 2026 | Repo from zero, CI, Docker Compose, WSL2 work. `AI_RULES.md`, `AI_BRIEF.md` and the contract (types, traits, errors, migration 0001) are written and frozen. Small Rust "hello" with one test. |
| M1 | Safe core: inventory, reservation, idempotency | Jan-Feb 2027 | Property tests for I1, I2, I4 pass on real Postgres. |
| M2 | Queue, random-window policy, SSE, minimal UI | Mar-Apr 2027 | One event works end to end. I3 tested with a restart. |
| M3 | Simulator, baseline, adaptive controller | May-Jun 2027 | Scenario A runs with one command. Controller compared with baseline. |
| M4 | Fault injection, k6, metrics endpoint, report | Jul 2027 | Scenario C passes. README and final report written. |

Risk: M1 and M2 are the hardest because they mix Rust learning with concurrency.
Risk (new): three AIs can drift apart in names, types, and error style. The contract and `AI_RULES.md` exist to stop this.
Risk (new): free AI plans have usage limits. Plan the weekly work in small chats.

Cut list: FIFO, multi-event and Grafana are already outside v1.0 (D1). So this file has no more safe cuts. If time runs short, you decide a new cut and write it in the decision log. Never cut the invariant tests.

## 2. Main idea

Two separate jobs:

- **Policy** decides the ORDER of users (random entry in a window in v1.0, FIFO in v1.2).
- **Controller** decides the SPEED of admission (how many users per second enter).

They are different modules. This lets us compare baseline and adaptive controller with the same policy. Only the controller changes.

The domain logic is pure code: no database, no sockets, no real clock. The clock and the random number generator are passed in (with a seed). This gives us two things:

- The simulator can reuse the same domain code.
- Tests are repeatable.

## 3. Modules (modular monolith)

| Module | One responsibility | Area |
| --- | --- | --- |
| `domain` | Pure types and rules: event, queue entry, reservation, order. No I/O. | A |
| `policy` | Orders users. Trait `AdmissionPolicy`. v1.0: `RandomWindow` only. FIFO comes in v1.2. | A |
| `controller` | Decides admission rate. Trait `RateController`. First: `FixedTokenBucket` (baseline). Then `Adaptive`. | A |
| `sim` | Separate crate. Discrete-event simulator using `domain`, `policy`, `controller`. | A |
| `event_owner` | One Tokio task per event. The only writer for that event. v1.0 runs one event. | B |
| `inventory` | Reservations with expiry, purchases, counts. | B |
| `idempotency` | Stores result per key, returns the same result on retry. | B |
| `payment_mock` | Fake payment: 200 ms latency, 5% failures (configurable). | B |
| `store` | SQLx and Postgres. Migrations. | B |
| `api` | Axum routes, SSE, rate limit, session deduplication. | C |
| `metrics` | `tracing` setup and a Prometheus-format metrics endpoint. No dashboard in v1.0. | C |

Rule: `domain`, `policy` and `controller` must not depend on `store` or `api`. If you break this, the simulator stops working.

Critical modules (Document 0, section 15): `domain`, `policy`, `controller`, `event_owner`, `inventory`, `idempotency`. They get the strict `unwrap` and `expect` lint.

## 4. One writer per event

Idea: each event has one Tokio task that owns its queue state. All commands (join, leave, admit, reserve, buy) go to it through a channel. It handles them one by one. So there are no races inside one event, and no locks.

Java comparison: it is like one single-thread executor per event, with a `BlockingQueue` of commands.

Important rules:

- Postgres is the source of truth. Memory is only a cache of it.
- The owner writes to Postgres BEFORE it answers the client. (This protects I3.)
- On restart, each owner rebuilds its state from Postgres.
- The database must also protect itself. We do not trust the owner alone (see section 5).
- Different events run in different tasks. v1.0 has one event. The design already allows many, so v1.1 (multi-event, Scenario B) is cheap to add.

Open question: the owner is a bottleneck for one event. This is the measured limit we report. We do not fix it before we measure it.

## 5. Data model that protects the invariants (sketch)

Minimal tables. Names can change before the contract is frozen.

| Table | Key points |
| --- | --- |
| `events` | `total_inventory`, `policy`, `window_start`, `window_end`. |
| `queue_entries` | `UNIQUE (event_id, session_id)`. Holds the random rank. |
| `reservations` | `UNIQUE (event_id, user_id)` for active reservations. Has `expires_at`. |
| `orders` | One per completed purchase. |
| `idempotency_keys` | `PRIMARY KEY (key, operation)`. Stores the response. |

How each invariant is protected:

- **I1 (no overselling):** inventory counts live in Postgres. A `CHECK (sold + reserved <= total)` rule. The reserve step changes the count and inserts the reservation in ONE transaction. If it fails, nothing changes.
- **I2 (no double admission/reservation):** unique constraints in the database. The owner also checks, but the constraint is the last defence.
- **I3 (no loss after restart):** queue entries and reservations are saved before the answer. Expiry uses `expires_at` timestamps, not timers in memory.
- **I4 (idempotency):** the idempotency row is written in the SAME transaction as the effect. A retry reads the stored result.

## 6. Main flows

**Join:** client sends join, owner stores queue entry, client gets position via SSE.

**Window closes:** policy assigns the order (random with a seed that is saved, so the result can be reproduced and audited).

**Admission:** controller gives the rate. Owner admits users at that rate.

**Reserve and buy:** admitted user reserves (with expiry), then pays (mock), then order is created. An expired reservation returns its inventory.

**Controller input:** latency and error rate of the protected backend. Output: admission rate. In the simulator, the protected backend is a model with limited capacity. In real tests, it is the reserve and buy endpoints.

## 7. Controller (first idea, not a decision)

Start with AIMD: slowly raise the rate when latency is good, cut it fast when latency or errors are bad. It is the same idea as TCP congestion control. It is simple, explainable and testable. We choose it with an ADR after we see the first baseline results.

Safety rule: the controller only changes SPEED. It can never change the order, and it can never touch inventory.

## 8. Tests

- `proptest`: random command sequences, then check I1-I4 after each step.
- Re-run the invariant checks after every fault injection (kill, DB down, slow network).
- Simulator: runs with a seed. Same seed gives same result.
- Playwright: one happy path in the UI.
- Jest and React Testing Library: queue and purchase components.
- k6: independent check on real HTTP.
- Who writes tests: for critical code, a different AI than the code author (Document 0, section 11).

## 9. Open questions (need your decision)

1. How long is the opening window? Scenario A has arrivals for 10 minutes. What happens to users who arrive after the window closes? (They go to the end in arrival order? They are rejected?)
2. What is a "user" for deduplication: a session cookie only? (Document 0 says basic anti-bot only.)
3. What exactly is the "protected backend" in real tests: only the reserve endpoint, or reserve and buy?
4. Expiry cleanup: a background job in the owner, or checked when needed?
5. Cargo workspace layout (crates) and folder structure. This is decided in the scaffold step, before the three AIs start their areas. (The project restarts from zero, D5.)

## 10. Not in this file (on purpose)

- ADRs (after discovery).
- Any number about performance. We have no measurements yet.
- FIFO, multi-event and Grafana designs (v1.1, v1.2, v1.3).

## 11. How the three AIs work independently (added by D5)

**Idea.** Three AIs write code in parallel. They do not talk to each other. They only share files. So the shared files must be strict and small.

**What is shared (read by every AI):**

1. Document 0 and this Technical Document.
2. `docs/AI_RULES.md`: syntax, names, errors, logs, and the three areas.
3. The contract: types, traits, error types, and migration 0001. Files marked `CONTRACT: frozen`.

**Order of work:**

1. Scaffold: Cargo workspace, folders, Docker Compose, CI, empty crates that compile (written by Claude, reviewed by Leonardo).
2. Contract: written by Claude, reviewed by ChatGPT and Gemini, approved and frozen by Leonardo.
3. Areas A, B, C start. Each AI works only inside its area.
4. Cross-review (Document 0, section 11) before each merge.

**Working without the others.** An AI that needs a module from another area codes against the trait in the contract. For tests it uses a small fake of that trait inside its own test code. Example: area B tests `event_owner` with a fake `AdmissionPolicy`, not the real one.

**Shared files.** Files where two areas meet (for example `main.rs`, the workspace `Cargo.toml`, `docker-compose.yml`) have ONE owner area and a big comment block at the top with small connection rules. Other AIs ask the owner through Leonardo.

**Contract change.** An AI that needs a change writes a change request (format in `AI_RULES.md`). Leonardo decides. If yes, the contract changes first, then all affected areas are updated.

**Integration.** Leonardo pastes code area by area. After each paste: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`. If something fails, the error goes back to the AI that owns the file.