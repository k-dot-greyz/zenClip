# Contributing to zenClip

Thank you for helping build **zenClip** — a cross-platform clipboard media monitor that deduplicates clips via SHA256, classifies content, emits MIDI events, and logs to a self-hosted REST API.

This repository is standalone product code. Keep contributions limited to implementation, tests, CI, and **product-facing** documentation.

---

## What belongs in this repository

| Allowed | Not allowed |
|---------|-------------|
| Rust core, API, bridge, and optional UI code | Internal monorepo guides, agent onboarding, switchboard registries |
| Tests, fixtures, and CI configuration | Parent-repo workflow copies (`SUBMODULE_*`, `AGENTS.md`, session notebooks) |
| Product docs (`README.md`, `LICENSE`, this file) | Fork-only runbooks or dex superproject SOPs |

Monorepo orchestration and submodule workflow docs live in the consuming organization's documentation repository, not here.

---

## Architecture expectations (GlitchWorks Agnostic)

zenClip is a **media processing pipeline**: clipboard capture → hash/dedup → content classification → MIDI emit → API persist. When you change behavior, align with these principles.

### Zero hardcoding (runtime configuration)

- MIDI CC mappings, API host/port, SQLite paths, ignore patterns, and `midi-gem` endpoints must come from **environment variables**, `zenclip.yaml`, or injected options — never hardcoded in domain logic.
- Platform-specific clipboard backends (`arboard`) stay behind a single monitor interface; do not scatter OS checks through classifiers or emitters.

### Polymorphism at pipeline boundaries

- Depend on abstractions, not concretions:
  - `IClipSource` — clipboard / file / replay input
  - `IDedupStore` — SHA256 ring / SQLite persistence
  - `IContentClassifier` — text / URL / image / audio detection
  - `IMidiEmitter` — CC/note output via `midi-gem` or local port
  - `IClipLogger` — REST API / stdout sink
- Domain orchestration must not import FastAPI, `arboard`, or HTTP clients directly; wire adapters at the edges.

### Open piping (typed events)

- Each pipeline stage emits **structured JSON events** on stdout or an internal bus — not shared mutable globals.
- Use a consistent envelope, for example:

```json
{
  "stage": "classifier",
  "sha256": "abc…",
  "content_type": "image",
  "outcome": "completed",
  "midi_cc": 22,
  "timestamp": "2026-06-04T12:00:00Z"
}
```

- Stages: `capture` → `dedup` → `classify` → `midi_emit` → `api_log`. Skipped dedup clips emit `outcome: "skipped"` with CC#24 semantics.

### Boundary validation (hostile edge)

- Validate clip payloads **before** hashing, classifying, or persisting:
  - Reject empty buffers, oversize blobs, and malformed URLs at the edge.
  - Normalize MIME hints; do not trust clipboard metadata alone for routing.
- API routes (`/clips`, `/clips/{sha}`, `/stats`) must validate path params, auth headers, and request bodies with explicit schemas before touching SQLite.
- MIDI bridge HTTP calls to `midi-gem` must validate response status and JSON shape before updating local state.

### Media format validation

- **Text / URL:** enforce max length, strip control characters, parse URLs with a strict parser; invalid URLs fall back to `text` or `rejected` — never panic.
- **Image:** verify magic bytes (PNG, JPEG, GIF, WebP) before treating as image; unknown formats → `binary` or skip with structured log.
- **Audio:** validate container headers where possible; unsupported codecs → `degraded` outcome, not crash.
- All validators live at the **pipeline edge**; classifiers receive already-sanitized metadata + byte slices.

### State hydration and dehydration

- Dedup store and in-memory ring must support `exportState()` / `loadState(payload)` (JSON or SQLite snapshot).
- Hydration must be **idempotent**: reloading the same snapshot must not duplicate MIDI emits or API rows.
- CLI and API startup should accept `--state-file` or equivalent for replay and crash recovery tests.

### Graceful degradation (predictable error recovery)

- Treat these as first-class outcomes, not panics:
  - Clipboard unavailable (headless / permission denied)
  - `midi-gem` offline or timeout
  - API bind failure or disk full on SQLite write
  - Duplicate clip (dedup skip → CC#24 value 0)
- On dependency failure: log structured error, emit telemetry, enter **safe idle** or **degraded mode** (capture + dedup only, no MIDI/API) when configured.
- Prefer result types (`{ ok, data?, error?, degradedFeatures? }`) over unhandled errors in orchestration code.
- Retry MIDI/API calls with bounded backoff + jitter; cap retries and surface final state to the caller.

### Agnostic telemetry

- Inject a logger interface; core logic must not assume journald, Docker, or a specific log shipper.
- Emit metrics as structured key/value pairs (`clips_captured`, `dedup_skips`, `midi_failures`, `api_latency_ms`).

---

## Repository layout

Current and target paths for the zenClip pipeline. Some directories are **planned** (see [tasks.md](tasks.md)); add rows here when scaffolding lands.

| Path | Purpose | Status |
|------|---------|--------|
| `README.md` | Product overview, architecture, MIDI mapping defaults | Present |
| `tasks.md` | Public roadmap / epic checklist | Present |
| `dex-entry.md` | Dex registry metadata (product index) | Present |
| `CONTRIBUTING.md` | This guide | Present |
| `crates/zenclip-core/` | Rust clipboard watcher, SHA256 dedup, JSON event emit | Planned |
| `config/zenclip.yaml` | CC mappings, filters, ignore patterns, endpoint URLs | Planned |
| `bridge/` | JSON → `midi-gem` HTTP adapter | Planned |
| `api/` | FastAPI server (`/clips`, `/clips/{sha}`, `/stats`) | Planned |
| `api/db/` | SQLite persistence layer and migrations | Planned |
| `ui/` | Optional Astro + TypeScript dashboard (SSE feed) | Planned |
| `tests/` | Unit, integration, and media fixture tests | Planned |
| `tests/fixtures/` | Sample clips (text, URL, image, audio) for validation | Planned |
| `.github/workflows/` | CI quality gates | Planned |

Runtime secrets and local state (API keys, SQLite files, replay snapshots) stay **gitignored** — never commit credentials or captured clipboard content.

---

## Development setup

Prerequisites (target stack from [README.md](README.md)):

- **Rust** (stable) — core monitor
- **Python 3.11+** — FastAPI API and `midi-gem` integration tests
- **Node.js 18+** — optional Astro UI
- **midi-gem** — [MIDI I/O microservice](https://github.com/k-dot-greyz/midi-gem) for local CC emit tests

```bash
git clone https://github.com/k-dot-greyz/zenClip.git
cd zenClip

# After scaffolding lands:
# cargo build --workspace
# pip install -e api/
# cp config/zenclip.example.yaml config/zenclip.yaml
```

Configure via `zenclip.yaml` and environment variables (names TBD during scaffold — document new vars in every PR).

---

## Quality gates

All pull requests should pass these gates before review. Run them locally on touched components.

### 1. Rust build and tests

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

- No new `clippy` warnings on touched crates.
- Unit tests for dedup ring, classifier edge cases, and event schema serialization.

### 2. CLI dry-run (no side effects)

```bash
zenclip monitor --dry-run --fixture tests/fixtures/
# or, before binary exists:
cargo run -- monitor --dry-run --fixture tests/fixtures/
```

- **Dry-run must not** open MIDI ports, write SQLite, or call external HTTP.
- Emit full JSON event stream to stdout for fixture replay.
- CI should run dry-run against committed fixtures on every PR.

### 3. Media format validation

```bash
cargo test media_validation
# or pytest tests/test_media_validation.py when Python validators exist
```

- Fixtures cover: valid/invalid URL, oversize text, each supported image magic, corrupt audio header, empty clipboard.
- Assert `outcome` is `completed`, `skipped`, `rejected`, or `degraded` — never process abort.

### 4. Error recovery tests

```bash
cargo test error_recovery
# API/MIDI integration tests with mocked offline dependencies
```

Required scenarios:

| Scenario | Expected behavior |
|----------|-------------------|
| Duplicate clip | Dedup skip, CC#24 = 0, no duplicate API row |
| `midi-gem` timeout | Bounded retry, then `degraded` log, monitor continues |
| API unavailable | Events buffered or dropped per config; no panic |
| Clipboard permission denied | Structured error, clean exit or idle loop |
| Invalid `zenclip.yaml` | Fail fast at startup with parse error message |
| State reload after crash | Idempotent hydration, no duplicate emits |

### 5. Python API checks (when `api/` exists)

```bash
ruff check api/
pytest api/tests/
```

- Schema validation on all routes; auth header required for mutating endpoints.

### 6. Optional UI (when `ui/` exists)

```bash
cd ui && npm ci && npm run build && npm test
```

---

## Development workflow

1. Fork the repository (or branch from `main` if you have write access).
2. Create a feature branch:

   ```bash
   git checkout -b feat/short-description
   ```

3. Make focused changes; keep commits small and [Conventional Commits](https://www.conventionalcommits.org/) compliant.
4. Run the quality gates above for every touched layer (Rust / Python / UI).
5. Push and open a pull request against `main`.

### Commit message format

```
type(scope): short description
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `ci`, `build`.

Examples:

- `feat(core): add SHA256 dedup ring with SQLite backend`
- `fix(classifier): reject oversize text at pipeline edge`
- `test(midi): add offline midi-gem recovery case`

---

## Pull request guidelines

1. Describe **what** changed and **why** (pipeline stage, MIDI mapping, API contract, or DX).
2. Link related issues when applicable.
3. Note new env vars, config keys, or breaking CLI flags.
4. Confirm no clipboard captures, API keys, or local SQLite DBs were committed.
5. Include dry-run output or test logs for media validation / recovery changes when helpful.

---

## Code style

- **Rust:** idiomatic error handling (`Result`), `thiserror` or equivalent at boundaries; keep I/O at crate edges.
- **Python:** type hints on FastAPI routes and pydantic models; validate at the HTTP edge.
- **General:** single-purpose functions; name pipeline stages consistently with event `stage` fields.
- Comment only non-obvious protocol details (MIDI CC mapping, clipboard platform quirks) — not what the code already states.

---

## Security and privacy

- zenClip processes clipboard content locally. Never log raw clip bodies in production telemetry unless explicitly opted in.
- API keys protect the self-hosted REST surface — do not commit or log them.
- Report vulnerabilities privately to the maintainers when possible.

---

## Getting help

- Open a GitHub issue for bugs or feature requests.
- Discuss large pipeline or MIDI contract changes in an issue before major refactors.

Thank you for contributing.
