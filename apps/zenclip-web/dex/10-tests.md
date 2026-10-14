---
dex_id: zenclip-web
dex_type: tests
status: prototype
tags: [tests, vitest]
owners: [human:operator]
---

# 10 — tests

Vitest covers:

- Engine interface / `SanitizeResult` / error `kind`s
- MOCK pass-through + WebCrypto sha256 (no golden hashes)
- wasm-bindgen wrapper mapping
- `loadEngine` MOCK fallback
- UI result/error copy
- File-type heuristic for upload fallback
- **`CaptureIsland` integration** (`ZCW-PR3-*` stories): mount, display capture, upload rejection, paste ablation, XSS-safe errors, object-URL revoke — see `docs/test-coverage-decision-log-pr3.md`

Run: `npm test` then `npm run typecheck` then `npm run build`.
