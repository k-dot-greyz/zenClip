# zenClip 🎹

> Clipboard media monitor → SHA256 dedup → MIDI → self-hosted API

Part of the **zenOS** ecosystem. MIDI as universal communication layer.

---

## Architecture

```
Clipboard Event (text/image/url/audio)
    → SHA256 dedup (skip if already seen)
    → Content classifier (type detection)
    → MIDI CC/Note emit (via midi-gem)
    → Self-hosted REST API log
    → Optional: trigger downstream zenOS workflows
```

## Stack

| Layer | Tech |
|---|---|
| Core monitor | Rust (`arboard` crate) |
| MIDI bridge | `midi-gem` (Python microservice) |
| Dedup store | SQLite / SHA256 hash ring |
| API | FastAPI (self-hosted) |
| Frontend (optional) | Astro + TypeScript |

## Features

- [ ] Cross-platform clipboard watcher (macOS, Linux, Windows)
- [ ] SHA256 dedup — no duplicates, ever
- [ ] MIDI CC emit on each new clip (configurable mapping)
- [ ] Content-type routing: text / URL / image / audio
- [ ] Self-hosted REST API with clip history
- [ ] zenOS MIDI integration via midi-gem
- [ ] Configurable via YAML (mappings, filters, ignore patterns)

## MIDI Mapping (default)

| Event | MIDI CC | Value |
|---|---|---|
| Text clip | CC#20 | length-normalised 0-127 |
| URL clip | CC#21 | domain-hashed 0-127 |
| Image clip | CC#22 | 127 (trigger) |
| Audio clip | CC#23 | 127 (trigger) |
| Dedup skip | CC#24 | 0 |

## Related

- [`midi-gem`](https://github.com/k-dot-greyz/midi-gem) — MIDI I/O routing microservice
- [`zenOS`](https://github.com/k-dot-greyz/zenOS) — parent operating system
- `dex_id`: TBD — tracked in [dev-master/dex](https://github.com/k-dot-greyz/dev-master/tree/main/dex)

## Status

`spec-complete` → `prototype-pending`

---

*Built for calm, sovereignty, and neurodivergent minds. No cloud. No noise. Just signal.*
