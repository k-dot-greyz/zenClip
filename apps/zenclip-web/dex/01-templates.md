---
dex_id: zenclip-web
dex_type: template
status: prototype
tags: [json, card, sanitize]
owners: [human:operator]
---

# 01 — templates

Sanitize result card (implementation-agnostic JSON). Bind UI to this shape; do not hardcode hashes.

```json
{
  "stage": "sanitize",
  "sha256": "64-lowercase-hex",
  "width": 1920,
  "height": 1080,
  "removedChunks": ["tEXt", "iTXt", "tIME"],
  "engine": "wasm | mock",
  "outcome": "completed"
}
```

Engine error envelope:

```json
{
  "stage": "sanitize",
  "outcome": "rejected",
  "kind": "decode | too_large | unsupported",
  "message": "human-readable"
}
```
