---
dex_id: zenclip-web
dex_type: scripts
status: prototype
tags: [scripts, npm]
owners: [human:operator]
---

# 04 — scripts

All commands run in `apps/zenclip-web/`.

| Script | Command |
|---|---|
| `dev` | `astro dev` |
| `build` | `astro build` |
| `preview` | `astro preview` |
| `typecheck` | `astro check` |
| `test` | `vitest run` |
| `test:watch` | `vitest` |

Wasm pkg (future, from repo root):

```
wasm-pack build crates/zenclip-wasm --target web --release
```
