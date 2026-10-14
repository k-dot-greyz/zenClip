---
dex_id: zenclip-web
dex_type: overview
status: prototype
tags: [overview, capture, sanitize]
owners: [human:operator]
---

# 00 — overview

zenClip web is the browser island for pixel capture.

Flow: `getDisplayMedia` → canvas → PNG bytes → `ZenClipEngine.sanitize` → show width, height, sha256, removed chunks.

Fallbacks: file upload, clipboard paste.

Out of scope: ingest, sink, MIDI, FastAPI, clipboard watchers.

Engine: typed binding in `src/engine/ZenClipEngine.ts`. Prefer `zenclip-wasm` (wasm-bindgen, `--target web`). If the pkg is missing, use the labelled MOCK pass-through (WebCrypto sha256).
