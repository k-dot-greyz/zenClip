---
dex_id: zenclip-web
dex_type: app
status: prototype
tags: [astro, vite, typescript, png, wasm, capture]
owners: [human:operator]
---

# zenClip web

Browser capture for zenClip. Display → canvas → PNG bytes, then the sanitize engine. File upload and paste are fallbacks. This app **does not** ingest, sink, or store clips.

Status lives on GitHub (issues, PRs, Projects). See [AGENTS.md](./AGENTS.md) for house rules.

## Run

From this directory:

```bash
npm install
npm run dev          # http://localhost:4321
npm run typecheck
npm test
npm run build
npm run preview      # CSP is enforced in preview/build, not `astro dev`
```

Node 22.12+. Only `PUBLIC_*` env vars are read. Copy `.env.example` if you need overrides.

| Command | What it does |
|---|---|
| `npm run dev` | Vite/Astro islands, hot reload |
| `npm run typecheck` | `astro check` (strict TS) |
| `npm test` | vitest, once |
| `npm run build` | static `dist/` |
| `npm run preview` | serve the build |

## Capture flow

1. **Display (main):** `getDisplayMedia` → one video frame → canvas → PNG bytes.
2. **File:** PNG bytes as-is; other still images rasterize through canvas to PNG.
3. **Paste:** clipboard image item, same as file.

The engine then `sanitize`s that buffer. The page shows **dimensions**, **SHA-256** (64 lowercase hex), and **removed chunks**. Ingest/sink is out of scope.

## Engine binding

`src/engine/ZenClipEngine.ts` is the contract:

- `sanitize(png: Uint8Array): Promise<SanitizeResult>`
- `sha256(bytes: Uint8Array): Promise<string>`
- `version(): string`

`SanitizeResult` is `{ png, sha256, width, height, removedChunks }`. Failures throw an `Error` with `kind: 'decode' | 'too_large' | 'unsupported'` and a message.

The JS methods are async because wasm init and the MOCK’s WebCrypto digest are async. Do **not** reimplement sanitize in JS. Do **not** pin golden hashes in tests.

### wasm-bindgen pkg (`zenclip-wasm`)

When the crate exists:

```bash
wasm-pack build crates/zenclip-wasm --target web --release
```

That emits a web-target pkg (JS glue + `.wasm`). Plug it in as:

1. **npm / git path** — add a dependency named `zenclip-wasm` pointing at the pkg (CI artifact tarball, `file:` path, or git URL).
2. **Env** — `PUBLIC_ZENCLIP_WASM` as a module specifier or URL if the pkg is not the default name.

`src/engine/wasm.ts` calls the wasm-bindgen `default()` init, then `sanitize` / `sha256` / `version`. If import or init fails, the app uses the labelled **MOCK**.

### MOCK

Pass-through. It peeks IHDR for width/height, hashes with WebCrypto, and reports `removedChunks: []`. The UI banner says `MOCK (pass-through)` so nobody mistakes it for the rust engine.

## Accessibility and CSP

- WCAG 2.2 AA targets: labels, live status, 44px hit areas, visible focus, skip link.
- `prefers-reduced-motion: reduce` kills animation and transitions.
- No runtime CDN scripts. Astro CSP hashes in `build` / `preview`. `blob:` is allowed for the PNG preview and capture video.

## Layout

```
src/engine/     contract, MOCK, wasm loader
src/capture/    display / file / paste → PNG bytes
src/ui/         result mapping + capture island
dex/            app dex (overview, templates, protocols, …)
docs/           screenshot and notes
```
