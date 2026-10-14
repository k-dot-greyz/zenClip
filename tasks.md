# zenClip — Tasks

The original clipboard-monitor / MIDI / FastAPI epics were written against a superseded spec. They are listed at the bottom as dropped.

## Epic: ZENCLIP-CAPTURE-001 — WYSIWYG pixel capture

- [x] `FrameSource` trait (full / region / window)
- [x] `MockCapture` for headless CI
- [x] `XcapCapture` adapter (Windows / macOS / Linux X11; Wayland caveats documented)
- [x] Yoink from file path, `http(s)` URL, and clipboard image
- [ ] Interactive region picker (out of scope for v0.1; pass `x,y,w,h`)
- [ ] Global hotkey (out of scope for v0.1)

## Epic: ZENCLIP-SANITIZE-001 — Metadata-free PNG

- [x] Re-encode from raw RGBA pixels (`png` crate)
- [x] Tag sRGB; do not copy source chunks
- [x] Strip EXIF / GPS / text chunks / tIME (tests with canaries)
- [x] Allow only IHDR, IDAT, IEND, sRGB, gAMA, cHRM
- [x] `sanitize_png_bytes` report (`png`, `sha256`, `width`, `height`, `removed_chunks`) for the web engine contract
- [ ] `crates/zenclip-wasm` wrapping that API (`wasm-pack --target web`)
- [ ] Formats other than PNG output (later, same pipeline)

## Epic: ZENCLIP-INGEST-001 — Hash, dedup, sinks

- [x] SHA256 of the sanitized PNG bytes; filename `{sha256}.png`
- [x] Inbox folder sink with dedup (skip if hash exists)
- [x] Optional sidecar JSON (never inside the PNG); fields configurable
- [x] Optional HTTP POST sink with documented generic contract
- [x] `IngestSink` trait left open for zenOS inbox (contract unknown — not invented)
- [x] `--dry-run` / `NullSink`

## Epic: ZENCLIP-CLI-001 — Interface

- [x] `zenclip shot --full|--region|--window`
- [x] `zenclip yoink <file|url|--clipboard>`
- [x] `zenclip windows`
- [x] YAML config + env overrides
- [x] JSON event on stdout

## Epic: ZENCLIP-WEB-001 — Browser capture UI

- [x] Astro island app in `apps/zenclip-web` (on `main`)
- [x] Engine contract `sanitize` / `sha256` / `version`; MOCK fallback
- [ ] Wire MOCK → `zenclip-wasm` once the crate exists
- [ ] Live `getDisplayMedia` verification on a real desktop session

## Epic: ZENCLIP-SNAP-001 — Bug snap mode

- [x] Card schema `config/schemas/bug-snap.card.json`
- [x] Care package example `config/care-packages/acme.example.json`
- [x] Architecture `docs/bug-snap.md`
- [ ] Sit *after* sanitize + hash (specimen is the clean PNG sha, not a clipboard body)
- [ ] Load packages at startup, fail on unknown matcher or class
- [ ] Matcher: host, content_type, prefix. First enabled wins
- [ ] Explicit `target_ref` override beats matcher
- [ ] Auto-fill pointer fields only. Judgment stays `pending_hand`
- [ ] No clip body persisted. Excerpt cap from package
- [ ] Snap MIDI CC from package if/when MIDI returns; not a v0.1 CLI concern
- [ ] Pair dance rejects a merged card

## Dropped (superseded spec)

- [x] ~~ZENCLIP-CORE-001 Rust clipboard-text monitor~~ dropped
- [x] ~~ZENCLIP-MIDI-001 midi-gem / CC map~~ dropped
- [x] ~~ZENCLIP-API-001 FastAPI `/clips` history~~ dropped
- [x] ~~ZENCLIP-UI-001 Astro MIDI dashboard / SSE clip feed~~ dropped (web capture UI is ZENCLIP-WEB-001)
