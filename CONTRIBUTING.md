# Contributing to zenClip

Thank you for helping build **zenClip** — a WYSIWYG media yoink: capture or re-render pixels, emit a metadata-free PNG, ingest by SHA256.

This repository is standalone product code. Keep contributions limited to implementation, tests, CI, and **product-facing** documentation.

The original clipboard→MIDI architecture notes in earlier revisions of this file are **superseded**. Match the pipeline below.

---

## What belongs in this repository

| Allowed | Not allowed |
|---------|-------------|
| Rust core, CLI, sinks, tests, CI | Internal monorepo guides, agent onboarding, switchboard registries |
| Product docs (`README.md`, `LICENSE`, this file) | MIDI / midi-gem / FastAPI clip-history leftovers |
| Fixtures that prove metadata is stripped | Captured inbox PNGs, API keys, live screenshots |

---

## Architecture expectations

Pipeline: **pixels → clean PNG → SHA256 → IngestSink**.

### Zero hardcoding

Inbox path, sidecar fields, HTTP sink URL, and yoink size limits come from `zenclip.yaml` or env (`ZENCLIP_*`), never from literals inside domain logic.

### Polymorphism at the edges

- `FrameSource` — screen / window / mock
- `IngestSink` — inbox folder / HTTP POST / dry-run / future zenOS inbox
- Decode/encode stay behind `decode_to_frame` / `encode_clean_png`

Do not import `xcap`, `arboard`, or `ureq` from `pipeline.rs`.

### Open piping

The CLI prints one JSON `ClipEvent` per run. Sidecar JSON is a separate file next to the PNG. Nothing goes into PNG text chunks.

### Sanitize is the product

Re-encode from samples. Do not copy PNG chunks or JPEG segments. Tests must keep proving EXIF/GPS/text/time cannot survive. If you add an encoder flag, add a test that would fail if it started writing `tEXt` or `tIME`.

### Privacy

No telemetry. No default remote host. URL yoink is an explicit user argument. HTTP ingest is opt-in.

---

## Repository layout

| Path | Purpose | Status |
|------|---------|--------|
| `README.md` | Product spec + quickstart | Present |
| `tasks.md` | Epic checklist | Present |
| `dex-entry.md` | Dex registry metadata | Present |
| `CONTRIBUTING.md` | This guide | Present |
| `crates/zenclip/` | Library + `zenclip` CLI | Present |
| `config/zenclip.example.yaml` | Inbox, sidecar, HTTP sink | Present |
| `.github/workflows/ci.yml` | fmt, clippy, tests | Present |

---

## Development setup

- Rust stable (MSRV 1.88)
- Linux live capture: X11 + `libxcb` headers. Wayland is unreliable (see README).

```bash
git clone https://github.com/k-dot-greyz/zenClip.git
cd zenClip
cp config/zenclip.example.yaml config/zenclip.yaml
cargo test -p zenclip
cargo run -p zenclip -- --mock --dry-run shot --full
```

---

## Quality gates

```bash
cargo fmt --all -- --check
cargo clippy -p zenclip --all-targets --all-features -- -D warnings
cargo test -p zenclip --all-features
cargo run -p zenclip -- --mock --dry-run shot --full
```

Required tests:

| Area | What must stay true |
|------|---------------------|
| Sanitize JPEG | EXIF + GPS canary gone; only allowed PNG chunks |
| Sanitize PNG | tEXt / zTXt / iTXt / tIME gone |
| Dedup | Second ingest of the same PNG is `duplicate`, one file |
| Sink | Inbox writes `{sha}.png`; sidecar optional and not in the PNG |
| Capture | `MockCapture` covers full/region/window without a display |

---

## Development workflow

1. Branch from `main`.
2. Conventional Commits: `feat(sanitize): …`, `fix(capture): …`, `test(ingest): …`.
3. Run the quality gates.
4. Open a PR against `main`. Do not commit inbox PNGs or secrets.

---

## Security and privacy

zenClip sees whatever is on the screen or in the image you pointed at. Never log raw frames. Report vulnerabilities privately to the maintainers when possible.
