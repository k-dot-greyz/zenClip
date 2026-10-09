# Bug snap mode

Clipboard specimen in, bug card out. The mode does not know ACME, Suno, or any other target. A care package supplies the target.

## Where it sits

```
capture → dedup → classify → bug_snap? → midi_emit → api_log
```

`bug_snap` runs only when mode is on and a package matcher hits. No match is a no-op. The clip still flows. Dedup skip does not re-snap.

## Auto context target

Packages live in `config/care-packages/*.json`. First enabled match wins. An explicit `target_ref` override beats the matcher.

Matcher kinds are config, seed set: `host`, `content_type`, `prefix`. Unknown matcher kind fails hydration. It is not coerced.

Auto fills the pointer: card id, sha256, target, droid, specimen ref, content type, time, pressure count, confidence `inferred`.

Hand fills the judgment: hold, break, class, zap action. Empty hold and empty break are legal. A vibe is not a value. Until the hand step lands, outcome is `pending_hand`.

Body text is not stored. Excerpt cap comes from the package. `store_body: false` is the default.

## Dances

| dance | when | step order |
|---|---|---|
| solo | one specimen | auto mint, hand drop, auto parse, hand judgment |
| pair | same prompt, two surfaces | hand names both, auto mints two cards, hand marks the delta, merge rejected |
| queue | abused droid | auto clusters pressure, hand confirms class once, auto stamps, hand overrides one or ships |

Zap default is `none` or `handoff`. A prompt rewrite is not the default for `user_abuse`.

## Event

Schema: `config/schemas/bug-snap.card.json`.

```json
{
  "stage": "bug_snap",
  "sha256": "<64 hex>",
  "card_id": "snap_01",
  "package_id": "acme-support",
  "dance": "queue",
  "target_ref": "acme",
  "droid_ref": "support.tier0",
  "pressure": { "kind": "repeat", "count": 3, "window": "PT24H" },
  "specimen": { "ref": "clip://<sha>", "content_type": "url" },
  "diagnosis": { "class": "unknown", "hold": "", "break": "", "confidence": "inferred" },
  "zap": { "action": "none", "status": "proposed" },
  "outcome": "pending_hand",
  "midi_cc": 25,
  "timestamp": "2026-10-09T01:11:00Z"
}
```

MIDI CC for snap is package config. It does not reuse CC#20–24.

## Copy a package

1. Copy `config/care-packages/acme.example.json`.
2. Set `package_id`, `target_ref`, `droid_ref`, matchers.
3. Set `enabled` true when the matcher is real.
4. Leave class enums in the file. Do not bake them into the monitor.
