---
dex_id: zenclip-web
dex_type: doc
status: prototype
tags: [tests, security, pr3]
owners: [human:operator]
---

# Test coverage decision log — PR #3 (Astro capture UI + MOCK engine)

Linked feat: [zenClip PR #3](https://github.com/k-dot-greyz/zenClip/pull/3) (merged).

## Stories implemented (MVP / 80–20)

| ID | Type | Impact | Scenario | Win condition |
|----|------|--------|----------|---------------|
| ZCW-PR3-UX-HAPPY-01 | UX | **High** | Operator opens capture page without wasm pkg | Engine line labels MOCK pass-through and states wasm is absent |
| ZCW-PR3-UX-HAPPY-02 | UX | Medium | Wasm engine loads in future | Compact `zenclip-wasm · version` line (no MOCK disclaimer) |
| ZCW-PR3-UX-HAPPY-03 | UX | **High** | Display capture happy path | Status → “Sanitize finished.”, result panel visible with dimensions / sha256 / chunks |
| ZCW-PR3-UX-SAD-01 | UX | **High** | Upload `.txt` or other non-image | User-facing rejection; sanitize not invoked (edge validation) |
| ZCW-PR3-UX-SAD-02 | UX | **High** | `getDisplayMedia` unavailable / denied | Button re-enabled; capture-kind message; result hidden |
| ZCW-PR3-UX-SAD-03 | UX | Medium | Truncated / bad PNG reaches engine | `[decode]` prefix in status (engine error channel) |
| ZCW-PR3-UX-ABLAT-01 | UX | Medium | Paste with no image clipboard items | Silent no-op; idle status (graceful ablation) |
| ZCW-PR3-UX-SAD-04 | DX | Medium | Broken partial DOM / bad island markup | `mountCaptureApp` throws “Missing capture UI node” |
| ZCW-PR3-SEC-01 | SEC | **High** | Hostile error string mimicking HTML (`<img onerror=…>`) | `textContent` only — no parsed `<img>` child; escaped `innerHTML` |
| ZCW-PR3-SEC-02 | SEC | Medium | Repeated captures (blob URL churn) | Prior `URL.revokeObjectURL` called before next preview |

### Security attack surface (predicted)

| Vector | Realistic scenario | Test |
|--------|-------------------|------|
| DOM XSS via error/capture messages | Malicious engine or browser error text echoed into status | SEC-01 |
| Blob URL memory / origin leakage | Operator captures repeatedly without navigation | SEC-02 |
| Clipboard MIME confusion | Non-image paste or empty items | AB LAT-01 (paste); file-type gate in existing `fallbacks.test.ts` |
| Oversize / decode bombs | PNG edge validation | Existing `pngPeek` / `mock` tests (not duplicated) |

## Deferred (follow-up issues)

- **Playwright smoke** on built `index.html` (real `getDisplayMedia` stubbed in browser) — not in CI yet; needs `astro preview` job or `@vitest/browser-playwright` once wasm pkg lands.
- **Paste happy path** with canvas rasterize — needs stable `Image` + PNG encode mocks in happy-dom; low ROI until paste bugs reported.
- **Agentic prompt injection** via pasted “instructions” in text clipboard — out of scope until ingest; document as non-goal in AGENTS.md.
- **Vitest `test.projects`** migration — replace deprecated `environmentMatchGlobs` when upgrading Vitest 4+ after npm arborist peer fix.

## Files

- `src/ui/CaptureIsland.test.ts` — island integration stories
- `src/ui/captureDomFixture.ts` — DOM fixture aligned with `index.astro`
- `vitest.config.ts` — happy-dom for island tests only
- `package.json` / lock — `happy-dom`; Vitest **3.2.4** (restores `npm ci` on Node 22.14; Vitest 4.0.x peer tree broke arborist)
