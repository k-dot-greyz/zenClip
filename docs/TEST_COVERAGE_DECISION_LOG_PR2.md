# Test coverage decision log — zenClip PR #2

**Feat (merged):** [PR #2 — v0.1 WYSIWYG PNG capture, sanitize, and inbox ingest](https://github.com/k-dot-greyz/zenClip/pull/2)

**Test PR:** (opened by coverage automation)

## Stories (80/20)

| ID | Type | Impact | Covered by |
|----|------|--------|------------|
| ZCLIP-PR2-CLI-01 | UX happy | **High** — CI/headless dry-run contract | `tests/pr2_ux_security_stories.rs` |
| ZCLIP-PR2-CLI-02 | UX sad | **Med** — malformed agent/operator region string | `tests/pr2_ux_security_stories.rs` |
| ZCLIP-PR2-SAN-03 | SEC sad | **High** — polyglot / prompt-injection prefix + PNG sig | `tests/pr2_ux_security_stories.rs` + `sanitize::decode_rejects_empty_and_non_image_bytes` |
| ZCLIP-PR2-ING-06 | SEC sad | **Critical** — path traversal via `record.sha256` | `ingest::rejects_path_traversal_in_content_hash` + `hash::is_valid_sha256_hex` |
| ZCLIP-PR2-ING-07 | SEC sad | **High** — wrong hash label on PNG body (swap attack) | `ingest::rejects_hash_mismatch_between_png_and_record` + `assert_content_addressed` |
| ZCLIP-PR2-PNG-08 | SEC sad | **Med** — hostile chunk length (DoS / parser safety) | `png_chunks::rejects_truncated_and_overflow_chunk_lengths` |
| ZCLIP-PR2-YOINK-09 | SEC sad | **High** — unbounded remote read | `yoink::yoink_url_rejects_oversize_payload` |
| ZCLIP-PR2-HTTP-10 | SEC sad | **Med** — misconfigured sink (enabled, empty URL) | `ingest::http_sink_rejects_enabled_without_url` |
| ZCLIP-PR2-PIPE-11 | UX ablation | **Med** — sidecar fields optional per config | `pipeline::sidecar_dimensions_can_be_ablated` |

## Attack surface notes

- **Inbox paths:** Filenames are derived from `sha256`. Without hex validation, `../` segments in a caller-supplied record could escape the inbox directory. Mitigation: `is_valid_sha256_hex` + byte/hash equality before any write.
- **HTTP yoink:** User-supplied URL + `max_yoink_bytes` is the trust boundary; oversized bodies must fail before decode.
- **PNG parser:** Untrusted yoink/file bytes; length fields must not panic or allocate unbounded buffers in our walker.
- **CLI region strings:** Treated as untrusted operator/agent input; parse errors must not emit ingest JSON (partial success).

## Deferred (low ROI / already covered elsewhere)

- Golden SHA256 pins (explicitly forbidden in CONTRIBUTING / web AGENTS.md).
- Live `xcap` capture (mock path covers pipeline).
- Full `zenclip-wasm` parity (crate planned).
- Playwright UI flows (web MOCK engine already has Vitest sad paths; `npm ci` blocked in this VM — follow-up).

## Documentation follow-ups (issues)

- [ ] `crates/zenclip-wasm` + web engine contract test matrix when WASM lands.
- [ ] Care-package / bug-snap matcher fail-fast tests when snap code ships (CONTRIBUTING table row).
- [ ] Document `assert_content_addressed` invariant in README ingest section.
