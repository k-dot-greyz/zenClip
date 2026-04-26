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
