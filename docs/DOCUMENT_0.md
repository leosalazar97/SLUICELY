# Document 0 — Non-negotiable project rules

Status: v0.4 · Working name: SLUICELY
This document overrides every other document. It can only be changed with an explicit, dated decision written in the decision log (section 16).

## 1. Purpose

Personal engineering and applied research project. It is not a product to sell and does not compete commercially with Queue-it or Cloudflare.
Goals, in priority order when they conflict:

1. Learn Rust, concurrent systems, Next.js/TypeScript, and testing in depth.
2. Produce a public repository that a senior engineer considers serious.
3. Have a narrow, measured, reproducible technical claim.

## 2. Project claim

"An admission engine for high-demand sales that protects the backend with an adaptive controller, validated by simulation, without ever violating inventory."

The claim is always split in two parts, never mixed:

- **Demonstrated**: what was measured, with hardware, scenario, and code version declared.
- **Estimated**: projections based on the measured bottleneck. They must be labeled as estimates and justified. A number without a model behind it does not enter the repository.

## 3. Audience and role

Audience: HR and engineers, including senior ones. Reference role: full stack, with the technical weight on the backend.
What the repo must convey: rigor (tested invariants, honest benchmarks, declared limits), not breadth of features.

## 4. What the software MUST do

Items without a note are in v1.0. Items with a note are moved to a later version by decision D1 (section 16).

1. Admit users to a limited resource (events with finite inventory) through a waiting queue.
2. Mandatory policy: random entry within an opening window. FIFO as a second policy, only after the first is complete. **FIFO moves to v1.2 (D1).**
3. Handle several events in parallel, each with its own queue, inventory, and policy. **Multi-event moves to v1.1 (D1). v1.0 has one event.**
4. Temporary inventory reservation with expiry, then purchase with a mocked payment.
5. Idempotency of critical operations (retry after timeout without double effects).
6. Update the queue state to the client in real time (SSE).
7. Adaptive controller: adjusts the admission rate based on latency and errors of the protected backend.
8. Load simulator (users, simple bots, reconnections, failures) used to validate and compare policies.
9. Demonstrable fault injection: process kill, database unavailable, slow network.
10. Minimal UI for queue and purchase, and a metrics endpoint. **The observability dashboard (Grafana) moves to v1.3 (D1).**
11. Rate limiting and session deduplication (basic anti-bot).

## 5. What the software will NEVER do

- Multi-tenant, billing, per-customer quotas.
- Real payments.
- Mobile app, email/SMS notifications, internationalization.
- Advanced admin.
- Advanced anti-bot (fingerprinting, ML anti-fraud).
- Kubernetes and complex distributed infrastructure.
- AI in the critical path. AI only decides forecasts and recommendations, never admissions, order, or inventory.
- Multi-language comparisons (Go/Java) as complete implementations.
- Paid services of any kind.

## 6. Non-negotiable invariants

If one of these is violated, the system is wrong, whatever the performance.

- I1: zero overselling (never more sales than inventory).
- I2: zero double admissions or double reservations of the same user for the same resource.
- I3: no loss of position or reservation after a restart.
- I4: every operation repeated with the same idempotency key produces the same result.

The invariants are verified by automatic (property-based) tests and re-run after every fault injection scenario.

## 7. Proof and validation

- Simple, documented baseline: random entry (same policy as the adaptive run) + fixed token bucket. Only the controller changes. Every result of the adaptive controller is compared with the baseline, in the same scenario. A FIFO comparison is a separate experiment, only after FIFO exists (v1.2).
- Metrics: throughput, latency p50/p95/p99, errors and timeouts, CPU and memory, DB connections, admission time, invariant violations (target: 0).
- Two levels of proof:
    - Discrete-event simulation: high scale (initial goal 50,000 simulated users or more), no real sockets.
    - Real load tests (k6 as independent verification): scale suited to the hardware, declared in the report.
- Every result reports: hardware, OS, code version, scenario, duration, repetitions. Numbers are reproducible with one command.
- On the current laptop, results are "indicative": they must be labeled with thermal and load conditions.
- Memory constraint: with 8 GB of RAM, Postgres, backend, frontend, and the observability stack do not run comfortably together with WSL2/Docker. During benchmarks, only the minimum is started (backend, Postgres, load generator). In v1.0 the backend only exposes a metrics endpoint. Prometheus is optional, and Grafana comes in v1.3. Both are used in dedicated sessions, not always on.
- The final report honestly declares the limits.

## 8. Stack (locked, no comparison at this stage)

| Area | Choice |
| --- | --- |
| Backend | Rust, Tokio, Axum, SQLx, Serde |
| Database | PostgreSQL, single source of truth. Redis only if a benchmark proves it is needed |
| Frontend | Next.js, TypeScript |
| Frontend tests | Jest, React Testing Library |
| Backend tests | cargo test, proptest (property-based) |
| End-to-end | Playwright |
| Load | Rust simulator + k6 (independent verification) |
| Realtime | SSE |
| Observability | tracing (Rust), metrics endpoint (v1.0), Prometheus optional (v1.0), Grafana (v1.3) |
| Infrastructure | Docker Compose, GitHub Actions |

Everything above is free or open source. Exceptions only with a written decision in section 16.

Small libraries are allowed under section 15 (Libraries rules).

## 9. Architecture rules

- Modular monolith: modules with clear responsibilities and boundaries.
- One owning writer per event (partitioning by event): no distributed consensus.
- Correctness does not depend on non-deterministic components.
- Every added component must have a written responsibility. No technology "for decoration".
- Architecture decisions are recorded in short ADRs, written after the discovery, not before.

## 10. Real constraints

- People: one.
- Time: 3-4 hours per week (about 15 hours per month). MVP target date: 31 July 2027.
- Hardware: Acer Aspire A515-52G, Windows 11 Home, Intel i5-8265U (4 cores / 8 threads, 1.6 GHz base), 8 GB RAM (7.85 GB usable). Development planned with WSL2 or Docker.
- Budget: zero. This includes the AI tools: only free plans or free tiers.
- Starting level: no experience in Rust, Next.js, Jest. Previous experience in JavaScript, Java, Spring Boot, MySQL, Supabase.

## 11. Working rules (changed by D5)

The project starts from zero. No code from earlier tools is reused.

**Roles**

- **Leonardo (CTO):** decides, owns Document 0, reads and understands every line. He is the ONLY one who copies code into the repo and pushes to GitHub.
- **Three AI writers:** Claude, ChatGPT, Gemini. They write all the code. They never touch the repo directly. They give code to Leonardo as text.
- **No coding agents or auto-writing tools.** Nothing writes to the repo except Leonardo.

**Areas (see AI_RULES.md for the full detail)**

| Area | Content | AI |
| --- | --- | --- |
| A, pure core | `domain`, `policy`, `controller`, `sim` | ChatGPT |
| B, data and concurrency | `store`, `inventory`, `idempotency`, `event_owner`, `payment_mock` | Claude |
| C, edges | `api`, `metrics`, Next.js UI, Playwright, Jest, k6, Docker, CI | Gemini |

An area can be moved to another AI only with a written decision in section 16.

**Independence**

- Each AI works inside its own area and can work without the others, using the contract.
- `AI_RULES.md` holds the shared rules (syntax, names, errors, logs). It is the communication file between the AIs.
- Files where two areas meet start with a big "shared file" comment block with small connection rules (template in `AI_RULES.md`).
- **Contract first:** shared types, traits, error types, and the first migration are written and frozen before the areas start. Only Leonardo can approve a contract change. An AI never changes the contract alone. It writes a change request.

**Review**

- Critical code is reviewed by an AI that did not write it: A is reviewed by B's AI, B by C's AI, C by A's AI.
- Tests for critical code are written by an AI other than the author of the code.
- Reviews look first for broken invariants, race conditions, and missing tests.

**Merge rule:** nothing enters the repo if it does not compile, if tests do not pass, if it has no comments and logs as required by section 15, or if Leonardo cannot explain it.

**Commits:** one AI, one area, one idea per commit. The commit message starts with the area tag: `[A]`, `[B]`, `[C]` or `[contract]`.

## 12. Repository and documentation

- Public GitHub, MIT or Apache-2.0 license.
- README readable in 3 minutes: what it is, claim, results, how to reproduce them.
- One architecture file and short ADRs. No long or noisy documentation.
- `docs/AI_RULES.md` and `docs/AI_BRIEF.md` are part of the repo.
- Every file has one responsibility. Significant functions document purpose, errors, and guarantees.
- CI with automatic tests on every push.

## 13. Closed decisions and open points

Closed:

- Language: everything in simple English (code, comments, commits, README, ADRs, chat with the AIs, and this Document 0). The AIs always answer in English. If I write a term in Italian it has to be translated so I can learn it. Only private notes stay in Italian.
- MVP target date: 31 July 2027, with checkpoints M0-M4 (defined in the technical document, section 1).
- Versions: v1.0, v1.1, v1.2, v1.3, v2 (see D1 in section 16).
- Working method: three AIs, Leonardo controls and pushes (see section 11, D5).
- Reference scenario: see section 14.
- Coding rules: see section 15.

Still open:

- Folder structure of the repo (to be decided in the scaffold step, before the areas start).
- WSL2 and Docker setup for 8 GB of RAM (setup phase).

## 14. Reference scenario (initial values, can be changed after the first measurement)

Scenario A, "Presale" (v1.0):

- 1 event, 5,000 tickets, 50,000 simulated users.
- Sale opens at T0. 60% of users arrive in the first 30 seconds, the rest in the following 10 minutes.
- Policy: random entry within an opening window.
- Temporary reservation of 5 minutes. 70% of those who reserve complete the purchase in 1-3 minutes.
- Mocked payment: 200 ms latency, 5% random failures.
- 10% of users are simple bots that send 10 times more requests.
- Protected backend with limited capacity (value to be set in the simulator), whose latency gets worse when capacity is exceeded.

Scenario B, "Multi-event" (v1.1, see D1): 3 events at the same time with different inventories and policies. Note: different policies need FIFO, which comes in v1.2. In v1.1, the 3 events use the random-window policy with different inventories. Different policies are tested in v1.2.

Scenario C, "Failures" (v1.0): in scenario A, process kill at T0+2 min, database unavailable for 30 s, slow network (500 ms) for 1 minute.

Real tests with k6 (indicative, on the laptop): from 2,000 to 5,000 virtual users. The measured breaking point is reported.
Projection to larger scales (e.g. 200,000) is an estimate based on the measured bottleneck, labeled as an estimate.

Success criteria:

- v1.0: 0 violations of I1-I4 in scenarios A and C.
- v1.1: 0 violations of I1-I4 also in scenario B.
- The adaptive controller improves at least one metric over the baseline (timeouts or admission time p95) in the same scenario.
- Every result is reproducible with one command.

## 15. Coding rules

**Comments** (they exist so that you learn):

1. Every file starts with one comment: what the file does and what it contains.
2. Every function has a comment: what it does, which errors it can return, and what it guarantees. Tiny, obvious helpers can skip it.
3. Hard code gets a `LEARN:` comment. It explains in simple words what the code does and why. Use Rust or JS/TS terms only. Only Leonardo can remove a `LEARN:` comment.
4. All other comments explain the why.
5. Files where two areas meet start with a "shared file" comment block (see `AI_RULES.md`).

**Logs** (they exist so that you can see what is happening at that moment, added by D5 and D6):

1. Rust uses the `tracing` macros (`error!`, `warn!`, `info!`, `debug!`). No `println!` in production code.
2. Every important action writes a log line: state change, join, admission, reservation created or expired, purchase, idempotency hit, controller rate change, restart recovery, every error.
3. Fixed levels: `error` = something is broken, `warn` = unusual but handled, `info` = important business event, `debug` = detail for development.
4. Logs use named fields, not only text: `event_id`, `session_id`, `user_id`, `idempotency_key`, `reservation_id`.
5. Never log secrets (passwords, tokens, full cookies).
6. Logs never change a result. Removing every log line must not change behavior.
7. No logs inside simulator hot loops. The simulator logs only summaries and important events.
8. TypeScript uses one small logger helper (no bare `console.log` in committed code).

**Rust:**

1. `cargo fmt` passes. `cargo clippy -- -D warnings` passes (every warning fails CI).
2. No `unsafe`.
3. `unwrap()` and `expect()` are controlled by the Clippy lints `unwrap_used` and `expect_used`:
    - Tests: allowed freely.
    - Production code: warning, so CI fails. The only exception is a hard problem with no other fix. Then the line gets `#[allow(clippy::unwrap_used)]` and a `// UNWRAP:` comment with the reason, and Leonardo approves it.
    - Critical code: error, no exceptions.

**TypeScript:**

1. Strict mode on, no `any`. ESLint and Prettier pass.

**Critical code** (inventory, queue, reservation, idempotency, admission):

1. It must have a test. A different AI must review it before merge, and a different AI must write its tests.
2. `domain`, `policy`, and `controller` never read the real clock, never call random directly, and never do I/O. Time and random are passed in. `tracing` macros are allowed there (D6), because they never change results.

**Tests and database:**

1. Every bug gets a failing test first, then the fix.
2. Invariant tests are never skipped or deleted.
3. Only migrations change the schema. All times are UTC.

**Style:**

1. Clear full names. No short names like `qe` or `res2`. Rust uses `snake_case`, TypeScript uses `camelCase`.
2. One responsibility per file. If a function needs a long explanation, split it.
3. Small commits, in English, one idea each.

**Libraries:**

1. Small, free libraries are allowed (for example `thiserror`, `uuid`). Each one needs a one-line reason in the commit.
2. They never replace the locked stack (Tokio, Axum, SQLx, Serde, Next.js).
3. They never hold the logic of critical code.

**The three AIs:**

1. Every AI gets `AI_BRIEF.md` at the start of every chat (it points to Document 0, the Technical Document, `AI_RULES.md`, and the contract).
2. Every AI delivers full files with their path, comments, logs, and tests, plus a short explanation of the critical parts.
3. The merge rule (section 11) stays: it compiles, tests pass, comments and logs are there, and Leonardo can explain it.

### The Clippy settings (for M0, when the repo exists)

These are for later. They are written here so the rule is clear.

- **Workspace `Cargo.toml`:** set `unwrap_used` and `expect_used` to `warn`.
- **`clippy.toml`:** set `allow-unwrap-in-tests` and `allow-expect-in-tests` to `true`.
- **Top of each critical module file:** `#![deny(clippy::unwrap_used, clippy::expect_used)]`. A `deny` turns the warning into a build error. Critical modules are `domain`, `policy`, `controller`, `event_owner`, `inventory`, `idempotency`.
- **CI:** run `cargo clippy -- -D warnings`.

## 16. Decision log

- **2026-10-22, D1:** Versions. v1.0 has one event, random-window policy, baseline and adaptive controller, simulator, fault injection, k6, and invariant tests. Only the metrics endpoint, no dashboard. Multi-event moves to v1.1, FIFO to v1.2, Grafana to v1.3. v2 is decided after the v1.0 measurements. This changes section 4 (items 2, 3, 10) and the Scenario B success criteria.
- **2026-10-22, D2:** Comment rules replace "comment the why, not the what" (section 15, Comments).
- **2026-10-22, D3:** Small libraries are allowed under section 15 (Libraries).
- **2026-10-22, D4:** `unwrap` and `expect` are enforced by Clippy lints (section 15, Rust rule 3).
- **2026-11-13, D5:** Working method. OpenCode and all auto-writing tools are removed. Claude, ChatGPT and Gemini write all the code, split in three areas (A, B, C). Leonardo reviews, copies the code into the repo, and pushes. The project restarts from zero. Contract first. Cross-review and tests by a different AI. Logs are required (section 15). Product decisions, scope, invariants, stack and scenarios do not change. This changes sections 11, 12, 13 and 15.
- **2026-11-13, D6:** `tracing` macros are allowed in `domain`, `policy` and `controller`. They are not real I/O from the module's point of view and never change results. Not allowed in simulator hot loops. This refines section 15, Critical code rule 2.
- **2026-11-13, D7:** New files `docs/AI_RULES.md` (shared rules and the three areas) and `docs/AI_BRIEF.md` (text pasted at the start of every AI chat). Open point "folder structure" is closed in the scaffold step.

TECHNICAL DOCUMENT

TODO LIST

AI_RULES

AI_BRIEF