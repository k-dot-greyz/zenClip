---
dex_id: zenclip-web
dex_type: project
status: prototype
tags: [project, github]
owners: [human:operator]
---

# 08 — projects

Status for this app lives on GitHub issues, PRs, and Projects — not in this file.

## User story (MVP)

As an operator I open zenClip web, capture a display (or upload/paste an image), and see the sanitize result: pixel size, SHA-256, and which PNG chunks were removed. I can tell whether I am on MOCK or wasm. I cannot ingest anywhere from this page.

UX flow:

1. Land on `/`. Engine banner states MOCK or wasm.
2. Capture display, or upload, or paste.
3. Live status updates without trapping focus.
4. Result panel shows preview, dimensions, sha256, removed chunks.
5. Errors name `decode` / `too_large` / `unsupported` when the engine threw.

Out of scope in this MVP: sink, MIDI, account, crawl indexing (`robots.txt` disallows `/`).
