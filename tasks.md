# zenClip — Tasks

## Epic: ZENCLIP-CORE-001 — Rust clipboard monitor

- [ ] Scaffold Rust project (`cargo init`)
- [ ] Implement `arboard`-based clipboard watcher
- [ ] SHA256 hash + dedup ring (SQLite)
- [ ] Content-type detection (text/url/image/audio)
- [ ] Emit to stdout as JSON events

## Epic: ZENCLIP-MIDI-001 — MIDI bridge

- [ ] Wire JSON events to midi-gem HTTP endpoint
- [ ] Configurable CC mapping via `zenclip.yaml`
- [ ] Dedup skip → CC#24 emit

## Epic: ZENCLIP-API-001 — Self-hosted API

- [ ] FastAPI server: `/clips`, `/clips/{sha}`, `/stats`
- [ ] SQLite persistence layer
- [ ] Auth: API key header

## Epic: ZENCLIP-UI-001 — Dashboard (optional)

- [ ] Astro + TypeScript frontend
- [ ] Live clip feed via SSE
- [ ] MIDI visualizer panel

## Epic: ZENCLIP-SNAP-001 — Bug snap mode

- [x] Card schema `config/schemas/bug-snap.card.json`
- [x] Care package example `config/care-packages/acme.example.json`
- [x] Architecture `docs/bug-snap.md`
- [ ] Load packages at startup, fail on unknown matcher or class
- [ ] Matcher: host, content_type, prefix. First enabled wins
- [ ] Explicit `target_ref` override beats matcher
- [ ] Auto-fill pointer fields only. Judgment stays `pending_hand`
- [ ] No clip body persisted. Excerpt cap from package
- [ ] Snap MIDI CC from package, not CC#20–24
- [ ] Pair dance rejects a merged card
