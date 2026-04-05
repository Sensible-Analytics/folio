# Architectural Guardrails — Folio

> **Purpose**: Define non-negotiable architectural constraints for the Folio codebase.
> These guardrails protect the system's integrity as it evolves.

---

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| **Desktop Shell** | Tauri 2.x | `@tauri-apps/cli ^2.10.0` |
| **Backend (Rust)** | Rust 2021 Edition | Workspace: `apps/tauri`, `apps/server`, `crates/*` |
| **Frontend** | React + TypeScript | `^5.9.3` |
| **Build Tool** | Vite | — |
| **Styling** | Tailwind CSS | — |
| **State Management** | Zustand | — |
| **Charts** | Recharts | — |
| **Database** | SQLite (via Diesel 2.2) | Bundled |
| **Package Manager** | pnpm (monorepo) | Workspaces |
| **Testing** | Vitest + Playwright (E2E) | — |
| **Linting** | ESLint 9 + Prettier | — |

---

## Rust Workspace Structure

```
crates/
├── core/              # Domain models, business logic
├── market-data/       # Market data fetching & caching
├── connect/           # Bank connectivity adapters
├── storage-sqlite/    # Diesel-based SQLite storage layer
├── device-sync/       # Cross-device synchronization
└── ai/                # AI-powered features (categorization, insights)

apps/
├── tauri/             # Tauri desktop application (Rust + React)
└── server/            # Standalone server application
```

### Rust Guardrails

1. **No `unsafe` code** — `unsafe_code = "forbid"` in workspace lints.
2. **Clippy warnings are errors** — `all = "warn"` in workspace lints.
3. **Diesel for ORM** — All database access goes through Diesel, never raw SQL except migrations.
4. **SQLite only** — No external database dependencies. Data is local-first.
5. **Error handling** — Use `thiserror` for library errors, `anyhow` for application errors.
6. **Async runtime** — Tokio is the only async runtime. No mixing with other runtimes.
7. **HTTP client** — `reqwest` with `rustls-tls` only. No native TLS.

---

## Frontend Guardrails

1. **TypeScript strict mode** — No `any` types. Use proper type definitions.
2. **Component architecture** — Feature-based organization under `apps/frontend/src/features/`.
3. **State management** — Zustand for global state. No Redux, no Context for data state.
4. **Data fetching** — TanStack Query (React Query) for server state. No manual fetch calls.
5. **Styling** — Tailwind CSS utility classes only. No CSS modules, no styled-components.
6. **No direct DOM manipulation** — Use React refs when absolutely necessary.

---

## Tauri-Specific Guardrails

1. **IPC boundaries** — Frontend communicates with Rust only through Tauri commands (`#[tauri::command]`).
2. **Capabilities model** — Use Tauri v2 capabilities for permission scoping. No blanket permissions.
3. **No direct filesystem access from frontend** — All file I/O goes through Rust commands.
4. **Secrets in Rust only** — API keys, encryption keys, and credentials never touch the frontend.

---

## Data & Security Guardrails

1. **Local-first architecture** — All user data stored locally in SQLite. No cloud storage of financial data.
2. **Encryption at rest** — Sensitive data encrypted using `chacha20poly1305` + `x25519-dalek`.
3. **No analytics or tracking** — No third-party analytics, crash reporting, or telemetry.
4. **Secret scanning** — Pre-commit hooks scan for API keys, tokens, and credentials.
5. **Environment variables** — All configuration via `.env` files. Never commit `.env`.

---

## Monorepo Guardrails

1. **pnpm workspaces** — All packages managed through `pnpm-workspace.yaml`.
2. **Internal crate paths** — Rust crates referenced via `path = "crates/*"` in `Cargo.toml`.
3. **Shared TypeScript config** — `tsconfig.base.json` extended by all packages.
4. **Linting consistency** — ESLint config shared via `eslint.base.config.js`.

---

## Testing Guardrails

1. **Unit tests** — Co-located with source files (`*.test.ts`, `*_test.rs`).
2. **E2E tests** — Playwright for full application flows in `e2e/`.
3. **No test-only dependencies in production** — Dev dependencies only in `devDependencies`.
4. **Coverage threshold** — Maintain minimum 80% coverage on critical paths.

---

## Performance Guardrails

1. **Bundle size budget** — Frontend bundle must not exceed 500KB gzipped.
2. **Database queries** — All queries must use indexes. No N+1 queries.
3. **Market data caching** — Market data cached locally, never fetched synchronously on UI thread.
4. **Lazy loading** — Features loaded on demand. No eager loading of non-critical modules.

---

## Decision Records

All significant architectural decisions must be recorded as ADRs in `docs/adr/`.
See [ADR Template](docs/adr/TEMPLATE.md) for format.

---

## Diagrams

Architecture diagrams are maintained in `docs/architecture/diagrams/` using Mermaid syntax.
See [System Context Diagram](docs/architecture/diagrams/system-context.mmd) for the C4 overview.

---

## Enforcement

These guardrails are enforced through:

- **CI checks** — Linting, type-checking, and tests on every PR.
- **Code review** — Reviewers verify compliance with these guardrails.
- **Pre-commit hooks** — Secret scanning and formatting checks.
- **ADR process** — Any deviation requires an Architecture Decision Record.

---

*Last updated: April 2026 | Version: 3.0.0*
