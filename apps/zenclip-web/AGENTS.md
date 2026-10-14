---
dex_id: zenclip-web
dex_type: doc
status: prototype
tags: [agents, github, house-rules]
owners: [human:operator]
---

# AGENTS.md — zenClip web

This file is the canonical agent doc for `apps/zenclip-web/`. Repo-wide product rules still live in the root `CONTRIBUTING.md` and `dex-entry.md`.

## Status

Status lives on **GitHub**: issues, pull requests, and Projects. Do not invent a second tracker, a local status board, or a “source of truth” outside GitHub.

## House rules

1. **Ship through a PR.** Open a pull request. Do not merge it.
2. **greyZ’s explicit yes to merge.** A PR is not merged until greyZ (`@k-dot-greyz`) says yes in so many words. “LGTM” from an agent is not a merge.
3. **No secrets.** No API keys, tokens, captured PNG bodies in git, `.env` files, or clipboard dumps. Public config is `PUBLIC_*` only. `.env*` is gitignored; `.env.example` is the template.
4. **No silent creates.** Do not silently create issues, projects, labels, branches outside the working branch, or cloud resources. If something new is needed, say so in the PR.

## Scope for this app

- Work in `apps/zenclip-web/` (plus app-owned GitHub files such as CODEOWNERS / this app’s workflow).
- Do **not** touch Rust crates or the open WYSIWYG PNG pipeline PR.
- Do **not** reimplement the sanitize engine in JavaScript. Load `zenclip-wasm` or keep the labelled MOCK.
- Do **not** add ingest/sink. Capture + sanitize result only.
- Do **not** pin golden hashes.
- Do **not** put health claims in copy.

## Commands

```bash
cd apps/zenclip-web
npm run typecheck
npm test
npm run build
```
