# Contributing to zenClip

Thank you for helping build **zenClip** — a WYSIWYG media yoink: capture or re-render pixels, emit a metadata-free PNG, ingest by SHA256.

This repository is standalone product code. Keep contributions limited to implementation, tests, CI, and **product-facing** documentation.

The original clipboard→MIDI architecture notes in earlier revisions of this file are **superseded**. Match the pipeline below. Bug snap and the Astro web app sit beside it; they do not restore MIDI into the core.

---

## What belongs in this repository

| Allowed | Not allowed |
|---------|-------------|
| Rust core, CLI, sinks, tests, CI | Internal monorepo guides, agent onboarding, switchboard registries |
| `apps/zenclip-web`, snap schema / care packages | MIDI / midi-gem / FastAPI clip-history leftovers in new code |
| Product docs (`README.md`, `LICENSE`, this file) | Captured inbox PNGs, API keys, live screenshots |
| Fixtures that prove metadata is stripped | Parent-repo workflow copies (`SUBMODULE_*`, session notebooks) |

---

## Architecture expectations

Pipeline: **pixels → clean PNG → SHA256 → IngestSink** → optional **bug_snap?** (care-package match only).

### Zero hardcoding

Inbox path, sidecar fields, HTTP sink URL, and yoink size limits come from `zenclip.yaml` or env (`ZENCLIP_*`), never from literals inside domain logic.

Care-package target, droid, matchers, and class enums come from `config/care-packages/`. The card type does not name a company.

### Polymorphism at the edges

- `FrameSource` — screen / window / mock
- `IngestSink` — inbox folder / HTTP POST / dry-run / future zenOS inbox
- Decode/encode stay behind `decode_to_frame` / `encode_clean_png` / `sanitize_png_bytes`
- Snap matching, when implemented, is `ICarePackage` — do not bake targets into the CLI

Do not import `xcap`, `arboard`, or `ureq` from `pipeline.rs`.

### Open piping

The CLI prints one JSON `ClipEvent` per run. Sidecar JSON is a separate file next to the PNG. Nothing goes into PNG text chunks. Snap cards use `config/schemas/bug-snap.card.json` and store a ref + capped excerpt, not the PNG bytes.

### Sanitize is the product

Re-encode from samples. Do not copy PNG chunks or JPEG segments. Tests must keep proving EXIF/GPS/text/time cannot survive. If you add an encoder flag, add a test that would fail if it started writing `tEXt` or `tIME`.

The web engine in `apps/zenclip-web` must not reimplement sanitize in JS. It loads `zenclip-wasm` or falls back to a labelled MOCK.

### Privacy

No telemetry. No default remote host. URL yoink is an explicit user argument. HTTP ingest is opt-in. Care packages fail startup if a matcher kind or bug class is unknown — do not coerce unknowns to `other`.

---

## Repository layout

| Path | Purpose | Status |
|------|---------|--------|
| `README.md` | Product spec + quickstart | Present |
| `tasks.md` | Epic checklist | Present |
| `dex-entry.md` | Dex registry metadata | Present |
| `CONTRIBUTING.md` | This guide | Present |
| `docs/bug-snap.md` | Bug snap mode and care-package architecture | Present |
| `config/schemas/bug-snap.card.json` | Snap card schema | Present |
| `config/care-packages/` | Auto context target packages | Present |
| `config/zenclip.example.yaml` | Inbox, sidecar, HTTP sink | Present |
| `crates/zenclip/` | Library + `zenclip` CLI | Present |
| `crates/zenclip-wasm/` | wasm-bindgen wrapper for the web engine | Planned |
| `apps/zenclip-web/` | Astro capture UI (MOCK engine) | Present |
| `.github/workflows/ci.yml` | Rust fmt, clippy, tests | Present |
| `.github/workflows/zenclip-web.yml` | Web typecheck, test, build | Present |

Runtime secrets and local state (API keys, inbox PNGs, `config/zenclip.yaml`) stay **gitignored**.

---

## Development setup

- Rust stable (MSRV 1.88)
- Linux live capture: X11 + `libxcb` headers. Wayland is unreliable (see README).
- Node 22 for `apps/zenclip-web`

```bash
git clone https://github.com/k-dot-greyz/zenClip.git
cd zenClip
cp config/zenclip.example.yaml config/zenclip.yaml
cargo test -p zenclip
cargo run -p zenclip -- --mock --dry-run shot --full

cd apps/zenclip-web && npm ci && npm test && npm run typecheck
```

---

## Quality gates

Touched Rust:

```bash
cargo fmt --all -- --check
cargo clippy -p zenclip --all-targets --all-features -- -D warnings
cargo test -p zenclip --all-features
cargo run -p zenclip -- --mock --dry-run shot --full
```

Touched web:

```bash
cd apps/zenclip-web && npm ci && npm run typecheck && npm test && npm run build
```

Required Rust tests:

| Area | What must stay true |
|------|---------------------|
| Sanitize JPEG | EXIF + GPS canary gone; only allowed PNG chunks |
| Sanitize PNG | tEXt / zTXt / iTXt / tIME gone |
| Dedup | Second ingest of the same PNG is `duplicate`, one file |
| Sink | Inbox writes `{sha}.png`; sidecar optional and not in the PNG |
| Capture | `MockCapture` covers full/region/window without a display |
| Snap (when coded) | Unknown matcher fails fast; no match is a no-op; dedup skip does not mint a second card |

---

## Development workflow

1. Branch from `main`.
2. Conventional Commits: `feat(sanitize): …`, `fix(capture): …`, `test(ingest): …`.
3. Run the quality gates for every touched layer (Rust / web).
4. Open a PR against `main`. Do not commit inbox PNGs or secrets.

---

## Security and privacy

zenClip sees whatever is on the screen or in the image you pointed at. Never log raw frames. Snap cards store a ref and a capped excerpt, not the image body. Report vulnerabilities privately to the maintainers when possible.
