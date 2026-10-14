---
dex_id: zenclip-web
dex_type: protocol
status: prototype
tags: [protocol, wasm, capture]
owners: [human:operator]
---

# 02 — protocols

## Capture

1. Display: `navigator.mediaDevices.getDisplayMedia` → `<video>` → `canvas.drawImage` → `canvas.toBlob('image/png')`.
2. File: `File` → bytes. PNG signature → pass bytes. Else rasterize via canvas.
3. Paste: `clipboardData.items` with `image/*`.

Stop media tracks after one frame.

## Engine

```
loadEngine()
  → import PUBLIC_ZENCLIP_WASM or 'zenclip-wasm'
  → wasm-bindgen default() init
  → wrap sanitize / sha256 / version
  → else MOCK
```

wasm-pack command (crate lands later):

```
wasm-pack build crates/zenclip-wasm --target web --release
```

Throw `Error` with `kind` in `{ decode, too_large, unsupported }`.
