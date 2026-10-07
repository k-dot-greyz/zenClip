# zenClip

> What you see is what you get. Yoink the pixels, strip the metadata, drop a PNG in the inbox.

Part of the **zenOS** ecosystem. Local, quiet, no cloud.

---

## This is not the clipboard→MIDI spec

An earlier draft of this repo described a clipboard monitor that hashed clips and emitted MIDI CC via `midi-gem`. **That spec is superseded.** zenClip v0.1 is a WYSIWYG media-yoinking engine: it captures *exactly* what is on screen (or re-renders an existing image from its pixels), writes a metadata-free PNG, and hands that file to a content-management sink.

MIDI, midi-gem, CC mappings, the clipboard-text watcher, and the FastAPI clip-history API are out of scope.

---

## What it does

1. **Capture** — grab the pixels that are on screen *right then*: full monitor, a region (`x,y,w,h`), or a window. No chrome you did not see, no reconstructed layers, no “smart” crop.
2. **Yoink** — take an existing image (file path, `http(s)` URL, or clipboard *image*) and re-render it from decoded samples. The container is discarded.
3. **Sanitize** — encode a PNG from those raw pixels. No EXIF, XMP, IPTC, `tEXt`/`iTXt`/`zTXt`, `tIME`, GPS, software tags, thumbnails, or ICC blobs. Colour is tagged **sRGB** so it still looks the same. Anything you want to remember about the capture (hash, size, source kind, time) goes in a **sidecar JSON**, never inside the PNG.
4. **Ingest** — name the file `{sha256}.png`, skip it if that hash is already in the sink, otherwise write it. Default sink: a local inbox folder. Optional: HTTP POST. The zenOS inbox-ingestion pipeline is a documented trait, not a guessed API.

PNG first. Other formats later, same pipeline.

---

## Quickstart

```bash
git clone https://github.com/k-dot-greyz/zenClip.git
cd zenClip
cargo build -p zenclip --release

# Headless / CI: mock pixels, no display needed
./target/release/zenclip --mock --dry-run shot --full

# Real capture (needs a display)
./target/release/zenclip shot --full
./target/release/zenclip shot --region 40,80,800,600
./target/release/zenclip shot --window "Firefox"

# Re-render an existing picture (strips metadata)
./target/release/zenclip yoink ./photo.jpg
./target/release/zenclip yoink --clipboard

# List windows
./target/release/zenclip windows
```

Each command prints one JSON event on stdout:

```json
{"stage":"ingest","sha256":"…","outcome":"completed","width":800,"height":600,"source":"shot_full","path":"inbox/….png"}
```

Copy [`config/zenclip.example.yaml`](config/zenclip.example.yaml) to `zenclip.yaml` (or pass `--config`) to change the inbox path, sidecar fields, and optional HTTP sink. Env overrides: `ZENCLIP_CONFIG`, `ZENCLIP_INBOX_DIR`, `ZENCLIP_HTTP_SINK_URL`, `ZENCLIP_SINK_KEY`, `ZENCLIP_MAX_YOINK_BYTES`.

Global hotkeys are out of scope for v0.1.

---

## Architecture

```
pixels (screen | window | decoded file/url/clipboard)
        │
        ▼
  encode clean PNG (RGBA8 + sRGB, no ancillary metadata)
        │
        ▼
  SHA256 of those PNG bytes → {hash}.png
        │
        ▼
  IngestSink  ─┬─ InboxFolderSink   (default, local)
               ├─ HttpPostSink      (optional, generic POST)
               └─ NullSink          (--dry-run)
```

| Piece | Choice | Why |
|---|---|---|
| Language | Rust (MSRV 1.88) | Fast local CLI, no runtime, easy to test headless. |
| Capture | [`xcap`](https://crates.io/crates/xcap) behind a `FrameSource` trait | Maintained successor to `screenshots`; returns RGBA pixels; Windows / macOS / Linux X11. |
| Encode | `image` decode + `png` crate encode | We control every chunk. Re-encoding from samples is the sanitizer. |
| Tests | `MockCapture` | CI has no desktop. Live capture is optional. |

**Wayland:** xcap’s screen/window capture is incomplete on Wayland (portal / protocol gaps). X11, Windows, and macOS are the supported live-capture targets. Headless Linux still runs the rest of the pipeline via `--mock`.

**Privacy defaults:** local inbox, no telemetry, no cloud calls unless you pass an `https://` yoink URL or enable `http_post`. The HTTP sink is off by default and has no default host.

---

## Sanitize contract

Output PNG may contain only: `IHDR`, `IDAT`, `IEND`, `sRGB`, and optionally `gAMA` / `cHRM` (the sRGB set). Tests feed:

- a JPEG with an EXIF APP1 payload including a GPS IFD and a canary string
- a PNG with `tEXt`, `zTXt`, `iTXt`, and `tIME`

and assert the canaries and forbidden chunks are gone.

---

## Ingest sinks

```rust
pub trait IngestSink {
    fn contains(&self, sha256: &str) -> Result<bool>;
    fn put(&self, png: &[u8], record: &ClipRecord) -> Result<PutResult>;
}
```

`ClipRecord` is the sidecar (`{sha256}.json`): hash, optional width/height, source kind, optional unix timestamp. Never embedded in the PNG. Fields are individually switchable in YAML.

### HTTP POST (optional, generic)

This is **not** zenOS inbox ingestion. Assumed contract, so it can be replaced without folklore:

```
POST {url}
Content-Type: image/png
X-Zenclip-Sha256: <hex>
X-Zenclip-Width: <px>
X-Zenclip-Height: <px>
X-Api-Key: <ZENCLIP_SINK_KEY if set>

<raw PNG bytes>
```

2xx means stored. Response body ignored. Enabling `http_post` **replaces** the folder sink for that run (it does not dual-write). Leave it off to keep files local.

### zenOS inbox

The owner’s ecosystem has an inbox-ingestion pipeline. Its HTTP/filesystem contract is **not published here**, so zenClip does not pretend to speak it. Implement `IngestSink` when that contract exists.

---

## Status

`spec-complete` → **v0.1 prototype**

---

*Built for calm, sovereignty, and neurodivergent minds. No cloud. No noise. Just the pixels.*
