# medxz — Roadmap (Offline‑First EMR)

This document is a working tasklist to execute the product goals:
- Offline-first UX (local DB is primary UI source)
- Append-only operations + audit-first
- Multi-device support via sync (cloud now, edge hub later)

## Milestones

### M0 — Repo + Dev Environment
- [ ] Add a Rust server crate (Axum) under `server/` (or `crates/server/`) and keep `src-tauri/` for the desktop app.
- [ ] Add `docker-compose.yml` for Postgres + server (local dev).
- [ ] Add `.env.example` for server config (DB URL, JWT secret, storage paths).
- [ ] Add CI (format/lint/test) for Rust + TypeScript.
- [ ] Add scripts: `dev`, `migrate`, `seed`, `test` (Makefile or npm scripts).

### M1 — Contracts + Data Model (Backend-First)
- [ ] Define canonical identifiers:
  - `clinic_id`, `user_id`, `device_id`, `patient_id`, etc. as UUIDv7.
- [ ] Define an `Operation` envelope (idempotent, append-only):
  - `op_id`, `clinic_id`, `device_id`, `user_id`
  - `entity_type`, `entity_id`, `op_type`
  - `device_time`, `device_seq` (per-device monotonic counter)
  - `payload` (JSON), `schema_version`
  - `hash` (optional integrity), `signature` (future)
  - `server_received_at` (server-assigned)
- [ ] Define cursor + pagination strategy for pull:
  - Cursor as `(server_received_at, op_id)` or a server sequence (decide one).
- [ ] Define “critical data” merge rules (no silent overwrites):
  - Append-only: observations, meds events, notes revisions
  - Demographics: explicit merge policy + always-audited
  - Define “conflict record” representation for UI reconciliation
- [ ] Define attachment model + store-and-forward queue:
  - `attachment_id`, sha256, mime, size, created_by, local_path
  - Separate upload job state + retries
- [ ] Define audit log view requirements (query by patient/encounter + diff provenance).

### M2 — Server Backend (Axum + Postgres)
- [ ] Scaffold Axum service:
  - Config loader, structured logging (tracing), request IDs, health endpoints
  - SQLx pool + migrations
- [ ] Postgres schema/migrations (minimum viable):
  - `clinics`, `users`, `clinic_users`, `devices`
  - `ops` (append-only), indexes for `(clinic_id, cursor fields)`
  - `attachments` metadata
  - `sync_state` (per-device last ack / cursor)
- [ ] Authentication + tenancy:
  - Password-only auth (email+password, Argon2id), login endpoint
  - Session token issuance (bearer), clinic scoping
  - RBAC role claims (admin/clinician/front-desk)
- [ ] Sync endpoints (idempotent):
  - `POST /v1/sync/push` accept ops batch, dedupe by `op_id`
  - `GET /v1/sync/pull?cursor=&limit=` return ops page + next cursor
  - Validate clinic/device/user provenance against auth token
  - Store `server_received_at` and enforce stable ordering
- [ ] Attachments endpoints (separate from ops):
  - `POST /v1/attachments` multipart upload (local FS storage)
  - `GET /v1/attachments/{id}` download with authz
  - Storage abstraction to later swap FS → S3
- [ ] Admin/ops observability:
  - Per-device last sync time, backlog size, last error
  - Basic metrics/logging; optional `/metrics`
- [ ] Server tests:
  - Migration tests
  - Sync idempotency tests (replay-safe)
  - Cursor pagination correctness tests

### M3 — Desktop Backend (Tauri Rust + Local Encrypted SQLite)
- [ ] Local DB foundation:
  - Encrypted SQLite (SQLCipher) + migrations
  - WAL mode, crash-safe transactional writes
- [ ] Secret/key management:
  - Generate per-device DB key; store in OS secure storage (keychain/credential manager)
- [ ] Local domain tables + repositories:
  - Patients, encounters, observations, conditions, medications, notes, attachments metadata
- [ ] Operation log + audit:
  - Every write creates an `op` (append-only) + an audit entry in the same transaction
  - Idempotent apply of remote ops (dedupe by `op_id`)
- [ ] Merge/conflict engine:
  - Per-entity rules; never silent overwrite for critical data
  - Persist conflicts for UI resolution
- [ ] Sync client:
  - Background worker: push unacked ops, then pull missing ops
  - Cursor state persisted; retry/backoff; resumable on restart
  - “Sync now” command + status reporting
- [ ] Attachment queue:
  - Store-and-forward uploads; survive restarts; retries
  - Allow references to attachments before upload completes
- [ ] Desktop tests:
  - DB migration smoke tests
  - Idempotent op apply tests
  - Attachment queue durability tests

### M4 — Frontend MVP (React)
- [ ] Auth screens (login, clinic selection).
- [ ] Patient registration + search.
- [ ] Encounter creation + list.
- [ ] Observations entry + timeline.
- [ ] Conditions/diagnoses list + add.
- [ ] Medications: order + discontinue (event-ish timeline).
- [ ] Notes: create + revision history + compare.
- [ ] Attachments: capture/import, metadata, upload queue status.
- [ ] Audit/history views per patient/encounter.
- [ ] Conflict resolution UI (merge/reconcile + attribution).
- [ ] Sync status UI (last sync, backlog, errors) + “Sync now”.

### M5 — Security, Reliability, Operability
- [ ] RBAC enforcement on server (authoritative) + client-side UX gating.
- [ ] Encryption at rest for attachments (client-side) if required for threat model.
- [ ] Backup/restore:
  - Local encrypted backup export/import
  - Server DB backup procedure (pg_dump) for MVP
- [ ] Observability:
  - Structured logs everywhere
  - Sync error reporting + dashboards (MVP: admin endpoints)
- [ ] Threat model pass (PHI, device loss, token theft, replay).
- [ ] Load/scale basics:
  - Postgres indexes, query plans, pagination performance

### M6 — “Edge Hub Later” Readiness (Design-Time Constraints)
- [ ] Configurable sync target(s): cloud URL now, LAN hub later.
- [ ] Fallback strategy: try hub → cloud → offline.
- [ ] Protocol stays identical across targets; only base URL changes.
- [ ] Device routing policies (per clinic config).

### M7 — AI Integration (Later)
- [ ] Treat AI outputs as separate append-only artifacts (never silent edits).
- [ ] Provenance fields: model/version, prompt/input snapshot hash, user acceptance.
- [ ] On-device vs server AI decision + PHI handling policy.
- [ ] UX: suggestions + clinician sign-off workflow.

## Open Questions (Need Decisions)
- Offline login on a never-authed device: **No** (only previously authenticated sessions can be used offline).
- Shared devices: **Yes** via lock screen; **auto-lock after 1 hour** inactivity; **no user switching while offline**; **password unlock** (no biometrics for now).
- Attachment size limit for MVP: **25MB** (can revisit).
- Cursor strategy: **server-issued monotonic sequence** (recommended for stable ordering + resumable pulls).
- FHIR alignment: decide whether to model payloads as FHIR-like from day 1, or keep an internal model and add FHIR mapping/export later.
