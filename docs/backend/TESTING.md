# Testing

This repo uses layered tests:
- Unit tests (fast, deterministic)
- HTTP integration tests (Axum router in-process)
- DB integration tests (Postgres in Docker)
- Property tests for sync invariants
- Coverage gating for backend crates

## Local prerequisites
- Rust toolchain (`rust-toolchain.toml`)
- Node + pnpm
- Docker (required for Postgres-backed tests)

## Run backend tests (fast)
From repo root:
- `cargo test -p medxz-protocol -p medxz-server`

## Run frontend lint
From repo root:
- `pnpm lint`

## Run frontend build check
From repo root:
- `pnpm build`

## Run everything we gate on pre-push
From repo root:
- `pnpm verify`

## Run everything (DB + server + desktop app)
From repo root:
- `pnpm dev:all`

## Run Postgres-backed tests
1) Start Postgres:
   - `docker compose up -d db`
2) Run tests (when we add DB-backed tests, CI will run them automatically):
   - `DATABASE_URL=postgresql://medxz:medxz@localhost:5432/medxz cargo test -p medxz-server`

## Coverage
CI will run `cargo llvm-cov` for backend crates and fail the build if coverage drops below the configured threshold.

## Git hooks
This repo uses Husky for `pre-commit` and `pre-push` checks. After installing dependencies, run:
- `pnpm prepare`
