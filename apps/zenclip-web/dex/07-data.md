---
dex_id: zenclip-web
dex_type: data
status: prototype
tags: [data, png, env]
owners: [human:operator]
---

# 07 — data

This app does not persist clips. No inbox, no SQLite, no API.

Config (public only):

| Name | Role |
|---|---|
| `PUBLIC_ZENCLIP_WASM` | Module specifier/URL for the wasm-bindgen pkg |
| `PUBLIC_ZENCLIP_MAX_BYTES` | Reject PNG buffers over this size (`kind=too_large`) |

`.env*` is gitignored. `.env.example` is committed empty of secrets.

`public/robots.txt` is `Disallow: /` until launch.
