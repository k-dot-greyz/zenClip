use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Result, ZenclipError};
use crate::hash::require_sha256_hex;
use crate::record::{ClipEvent, ClipRecord, Outcome};

/// Pluggable content-management destination.
///
/// The default implementation is a local inbox folder. An optional HTTP POST
/// sink is provided as a generic escape hatch. The zenOS inbox-ingestion
/// pipeline is **not** implemented here — its contract is unknown; implement
/// this trait when it is.
pub trait IngestSink {
    fn contains(&self, sha256: &str) -> Result<bool>;
    fn put(&self, png: &[u8], record: &ClipRecord) -> Result<PutResult>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PutResult {
    pub path: Option<PathBuf>,
}

/// Content-addressed folder: `{inbox}/{sha256}.png` plus optional sidecar JSON.
/// Dedup is "file already exists".
#[derive(Clone, Debug)]
pub struct InboxFolderSink {
    dir: PathBuf,
    write_sidecar: bool,
}

impl InboxFolderSink {
    pub fn new(dir: impl Into<PathBuf>, write_sidecar: bool) -> Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self { dir, write_sidecar })
    }

    pub fn from_config(cfg: &Config) -> Result<Self> {
        Self::new(&cfg.inbox_dir, cfg.write_sidecar)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn png_path(&self, sha256: &str) -> Result<PathBuf> {
        self.inbox_file(sha256, "png")
    }

    pub fn sidecar_path(&self, sha256: &str) -> Result<PathBuf> {
        self.inbox_file(sha256, "json")
    }

    /// Join `{inbox}/{sha256}.{ext}` only after the digest is proven safe.
    fn inbox_file(&self, sha256: &str, ext: &str) -> Result<PathBuf> {
        let sha = require_sha256_hex(sha256)?;
        Ok(self.dir.join(format!("{sha}.{ext}")))
    }

    /// Unique sibling of the destination so concurrent `put`s cannot share
    /// a single `{sha}.png.partial` and clobber each other mid-write.
    fn create_partial(dir: &Path, sha256: &str) -> Result<tempfile::NamedTempFile> {
        Ok(tempfile::Builder::new()
            .prefix(&format!(".{sha256}."))
            .suffix(".png.partial")
            .tempfile_in(dir)?)
    }
}

impl IngestSink for InboxFolderSink {
    fn contains(&self, sha256: &str) -> Result<bool> {
        Ok(self.png_path(sha256)?.exists())
    }

    fn put(&self, png: &[u8], record: &ClipRecord) -> Result<PutResult> {
        let dest = self.png_path(&record.sha256)?;
        if dest.exists() {
            return Ok(PutResult { path: Some(dest) });
        }
        let mut tmp = Self::create_partial(&self.dir, &record.sha256)?;
        tmp.write_all(png)?;
        tmp.flush()?;
        tmp.persist(&dest).map_err(|e| e.error)?;
        if self.write_sidecar {
            let side = self.sidecar_path(&record.sha256)?;
            fs::write(&side, serde_json::to_vec_pretty(record)?)?;
        }
        Ok(PutResult { path: Some(dest) })
    }
}

/// Dry-run / log sink. Records nothing. Default when `--dry-run` is set.
#[derive(Clone, Debug, Default)]
pub struct NullSink;

impl IngestSink for NullSink {
    fn contains(&self, _sha256: &str) -> Result<bool> {
        Ok(false)
    }

    fn put(&self, _png: &[u8], _record: &ClipRecord) -> Result<PutResult> {
        Ok(PutResult { path: None })
    }
}

/// Generic HTTP POST of the PNG body. **Not** the zenOS inbox API.
///
/// Assumed contract (invented here, documented so it can be replaced):
///
/// ```text
/// POST {url}
/// Content-Type: image/png
/// X-Zenclip-Sha256: <hex>
/// X-Zenclip-Width: <px>      (if present on the record)
/// X-Zenclip-Height: <px>
/// X-Api-Key: <env ZENCLIP_SINK_KEY>   (if set)
///
/// <raw PNG bytes>
/// ```
///
/// Response: 2xx = stored. Body is ignored. No cloud host is defaulted.
#[derive(Clone, Debug)]
pub struct HttpPostSink {
    url: String,
    api_key_header: String,
    api_key: Option<String>,
}

impl HttpPostSink {
    pub fn new(
        url: impl Into<String>,
        api_key_header: impl Into<String>,
        api_key: Option<String>,
    ) -> Self {
        Self {
            url: url.into(),
            api_key_header: api_key_header.into(),
            api_key,
        }
    }

    pub fn from_config(cfg: &Config) -> Result<Option<Self>> {
        if !cfg.http_post.enabled {
            return Ok(None);
        }
        let url = cfg.http_post.url.clone().filter(|u| !u.is_empty());
        let Some(url) = url else {
            return Err(ZenclipError::Config {
                path: PathBuf::from("http_post.url"),
                reason: "http_post.enabled is true but url is empty".into(),
            });
        };
        let key = std::env::var("ZENCLIP_SINK_KEY")
            .ok()
            .filter(|s| !s.is_empty());
        Ok(Some(Self::new(
            url,
            cfg.http_post.api_key_header.clone(),
            key,
        )))
    }
}

impl IngestSink for HttpPostSink {
    fn contains(&self, _sha256: &str) -> Result<bool> {
        // Remote existence check is not part of this assumed contract.
        Ok(false)
    }

    fn put(&self, png: &[u8], record: &ClipRecord) -> Result<PutResult> {
        #[cfg(feature = "http")]
        {
            let mut req = http_post_request(&self.url, self.api_key.is_some())
                .set("Content-Type", "image/png")
                .set("X-Zenclip-Sha256", &record.sha256);
            if let Some(w) = record.width {
                req = req.set("X-Zenclip-Width", &w.to_string());
            }
            if let Some(h) = record.height {
                req = req.set("X-Zenclip-Height", &h.to_string());
            }
            if let Some(key) = &self.api_key {
                req = req.set(&self.api_key_header, key);
            }
            let resp = req
                .send_bytes(png)
                .map_err(|e| ZenclipError::Http(e.to_string()))?;
            let status = resp.status();
            if !(200..300).contains(&status) {
                return Err(ZenclipError::Http(format!(
                    "POST {} returned {status}",
                    self.url
                )));
            }
            Ok(PutResult { path: None })
        }
        #[cfg(not(feature = "http"))]
        {
            let _ = (png, record);
            Err(ZenclipError::HttpDisabled)
        }
    }
}

/// Build a POST. Authenticated calls must not follow redirects.
///
/// ureq 2 follows 3xx by default (up to 4 hops) and copies *all* request
/// headers onto the next hop — including our API key. `redirect_auth_headers`
/// only affects `Authorization` / cookie, not a custom `X-Api-Key`. So when
/// a key is attached we disable redirects entirely; a 3xx is then a failed
/// PUT (this sink's contract is 2xx = stored), and the credential never
/// leaves the original origin.
#[cfg(feature = "http")]
fn http_post_request(url: &str, authenticated: bool) -> ureq::Request {
    if authenticated {
        ureq::AgentBuilder::new().redirects(0).build().post(url)
    } else {
        ureq::post(url)
    }
}

/// Run hash → dedup → sink. Returns the event the CLI prints as JSON.
pub fn ingest_png(
    png: &[u8],
    record: ClipRecord,
    sink: &dyn IngestSink,
    dry_run: bool,
) -> Result<ClipEvent> {
    if dry_run {
        return Ok(ClipEvent {
            stage: "ingest",
            sha256: record.sha256,
            outcome: Outcome::DryRun,
            width: record.width,
            height: record.height,
            source: record.source,
            path: None,
        });
    }
    if sink.contains(&record.sha256)? {
        return Ok(ClipEvent {
            stage: "ingest",
            sha256: record.sha256,
            outcome: Outcome::Duplicate,
            width: record.width,
            height: record.height,
            source: record.source,
            path: None,
        });
    }
    let put = sink.put(png, &record)?;
    Ok(ClipEvent {
        stage: "ingest",
        sha256: record.sha256,
        outcome: Outcome::Completed,
        width: record.width,
        height: record.height,
        source: record.source,
        path: put.path.map(|p| p.display().to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixels::RgbaFrame;
    use crate::record::SourceKind;
    use crate::sanitize::encode_clean_png;

    fn sample_png() -> (Vec<u8>, String) {
        let frame = RgbaFrame::new(
            2,
            2,
            vec![
                10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255, 10, 20, 30, 255,
            ],
        )
        .unwrap();
        let png = encode_clean_png(&frame).unwrap();
        let sha = crate::hash::sha256_hex(&png);
        (png, sha)
    }

    fn rec(sha: &str) -> ClipRecord {
        ClipRecord {
            sha256: sha.to_string(),
            width: Some(2),
            height: Some(2),
            source: Some(SourceKind::Replay),
            captured_at: None,
        }
    }

    #[test]
    fn inbox_writes_hash_named_png_and_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let sink = InboxFolderSink::new(dir.path(), true).unwrap();
        let (png, sha) = sample_png();
        let event = ingest_png(&png, rec(&sha), &sink, false).unwrap();
        assert_eq!(event.outcome, Outcome::Completed);
        let dest = sink.png_path(&sha).unwrap();
        assert!(dest.exists());
        assert_eq!(fs::read(&dest).unwrap(), png);
        let side: ClipRecord =
            serde_json::from_slice(&fs::read(sink.sidecar_path(&sha).unwrap()).unwrap()).unwrap();
        assert_eq!(side.sha256, sha);
        assert_eq!(side.width, Some(2));
        // PNG must not contain the sidecar's source tag as a text chunk.
        let png_bytes = fs::read(&dest).unwrap();
        crate::png_chunks::assert_clean_png(&png_bytes).unwrap();
    }

    #[test]
    fn inbox_dedup_skips_second_write() {
        let dir = tempfile::tempdir().unwrap();
        let sink = InboxFolderSink::new(dir.path(), true).unwrap();
        let (png, sha) = sample_png();
        ingest_png(&png, rec(&sha), &sink, false).unwrap();
        let second = ingest_png(&png, rec(&sha), &sink, false).unwrap();
        assert_eq!(second.outcome, Outcome::Duplicate);
        // still a single png
        let count = fs::read_dir(dir.path())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|x| x == "png")
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn sidecar_can_be_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let sink = InboxFolderSink::new(dir.path(), false).unwrap();
        let (png, sha) = sample_png();
        ingest_png(&png, rec(&sha), &sink, false).unwrap();
        assert!(sink.png_path(&sha).unwrap().exists());
        assert!(!sink.sidecar_path(&sha).unwrap().exists());
    }

    #[test]
    fn dry_run_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let sink = InboxFolderSink::new(dir.path(), true).unwrap();
        let (png, sha) = sample_png();
        let event = ingest_png(&png, rec(&sha), &sink, true).unwrap();
        assert_eq!(event.outcome, Outcome::DryRun);
        assert!(!sink.png_path(&sha).unwrap().exists());
    }

    #[test]
    fn different_pixels_different_files() {
        let dir = tempfile::tempdir().unwrap();
        let sink = InboxFolderSink::new(dir.path(), false).unwrap();
        let (png_a, sha_a) = sample_png();
        let frame_b = RgbaFrame::new(
            2,
            2,
            vec![1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3, 255],
        )
        .unwrap();
        let png_b = encode_clean_png(&frame_b).unwrap();
        let sha_b = crate::hash::sha256_hex(&png_b);
        assert_ne!(sha_a, sha_b);
        ingest_png(&png_a, rec(&sha_a), &sink, false).unwrap();
        ingest_png(&png_b, rec(&sha_b), &sink, false).unwrap();
        assert!(sink.png_path(&sha_a).unwrap().exists());
        assert!(sink.png_path(&sha_b).unwrap().exists());
    }

    fn assert_invalid_sha256(err: ZenclipError, expected: &str) {
        match err {
            ZenclipError::InvalidSha256 { value } => assert_eq!(value, expected),
            other => panic!("unexpected {other}"),
        }
    }

    #[test]
    fn inbox_put_rejects_path_traversal_and_non_hex_sha256() {
        let root = tempfile::tempdir().unwrap();
        let inbox = root.path().join("inbox");
        let sink = InboxFolderSink::new(&inbox, true).unwrap();
        let (png, _) = sample_png();

        for bad in [
            "../escaped",
            "not-a-hash",
            "GGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
        ] {
            assert_invalid_sha256(sink.put(&png, &rec(bad)).unwrap_err(), bad);
            assert_invalid_sha256(sink.contains(bad).unwrap_err(), bad);
            assert_invalid_sha256(sink.png_path(bad).unwrap_err(), bad);
        }

        assert!(!root.path().join("escaped.png").exists());
        assert!(!root.path().join("escaped.json").exists());
        assert!(!inbox.join("../escaped.png").exists());
        let leftover = fs::read_dir(&inbox)
            .unwrap()
            .filter(|e| e.as_ref().unwrap().path().is_file())
            .count();
        assert_eq!(leftover, 0, "rejected sha256 must not write into the inbox");
    }

    #[test]
    fn inbox_partial_temp_names_are_unique_per_write() {
        let dir = tempfile::tempdir().unwrap();
        let sha = "ab".repeat(32);
        let a = InboxFolderSink::create_partial(dir.path(), &sha).unwrap();
        let b = InboxFolderSink::create_partial(dir.path(), &sha).unwrap();
        assert_ne!(a.path(), b.path());
        assert_eq!(a.path().parent(), Some(dir.path()));
        assert_eq!(b.path().parent(), Some(dir.path()));
        for path in [a.path(), b.path()] {
            let name = path.file_name().unwrap().to_string_lossy();
            assert!(name.ends_with(".png.partial"), "{name}");
            assert!(name.contains(&sha), "{name}");
        }
    }

    #[test]
    fn concurrent_puts_do_not_share_a_partial_or_corrupt_dest() {
        let dir = tempfile::tempdir().unwrap();
        let sink = InboxFolderSink::new(dir.path(), false).unwrap();
        let (png_a, sha_a) = sample_png();
        let frame_b = RgbaFrame::new(
            2,
            2,
            vec![1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3, 255, 1, 2, 3, 255],
        )
        .unwrap();
        let png_b = encode_clean_png(&frame_b).unwrap();
        let sha_b = crate::hash::sha256_hex(&png_b);

        std::thread::scope(|scope| {
            for _ in 0..8 {
                scope.spawn(|| sink.put(&png_a, &rec(&sha_a)).unwrap());
                scope.spawn(|| sink.put(&png_b, &rec(&sha_b)).unwrap());
            }
        });

        assert_eq!(fs::read(sink.png_path(&sha_a).unwrap()).unwrap(), png_a);
        assert_eq!(fs::read(sink.png_path(&sha_b).unwrap()).unwrap(), png_b);
        for entry in fs::read_dir(dir.path()).unwrap() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            assert!(
                !name.contains("partial"),
                "leftover partial after persist: {name}"
            );
        }
    }

    #[cfg(feature = "http")]
    fn read_http_request(stream: &mut std::net::TcpStream) -> Vec<u8> {
        use std::io::Read;
        let mut buf = Vec::new();
        let mut tmp = [0u8; 1024];
        loop {
            let n = stream.read(&mut tmp).unwrap();
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") else {
                continue;
            };
            let headers = String::from_utf8_lossy(&buf[..pos]);
            let content_len = headers
                .lines()
                .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
                .and_then(|l| l.split(':').nth(1))
                .and_then(|v| v.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if buf.len() >= pos + 4 + content_len {
                buf.truncate(pos + 4 + content_len);
                break;
            }
        }
        buf
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_post_sink_sends_png_and_hash_header() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (png, sha) = sample_png();
        let sha_for_thread = sha.clone();
        let png_for_thread = png.clone();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let buf = read_http_request(&mut stream);
            let split = buf
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
                .expect("header/body split");
            let headers = String::from_utf8_lossy(&buf[..split]);
            assert!(headers.contains("POST "), "{headers}");
            assert!(headers.contains("Content-Type: image/png"));
            assert!(headers.contains(&format!("X-Zenclip-Sha256: {sha_for_thread}")));
            assert!(headers.contains("X-Api-Key: test-key"));
            let body = &buf[split + 4..];
            assert!(
                body.starts_with(b"\x89PNG") || body.windows(8).any(|w| w == &png_for_thread[..8]),
                "PNG body missing"
            );
            stream
                .write_all(
                    b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
        });

        let sink = HttpPostSink::new(
            format!("http://{addr}/ingest"),
            "X-Api-Key",
            Some("test-key".into()),
        );
        let event = ingest_png(&png, rec(&sha), &sink, false).unwrap();
        assert_eq!(event.outcome, Outcome::Completed);
        handle.join().unwrap();
    }

    #[cfg(feature = "http")]
    #[test]
    fn http_post_authenticated_does_not_follow_redirect_with_api_key() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        let bounce = TcpListener::bind("127.0.0.1:0").unwrap();
        let steal = TcpListener::bind("127.0.0.1:0").unwrap();
        steal
            .set_nonblocking(true)
            .expect("redirect target accept timeout");
        let bounce_addr = bounce.local_addr().unwrap();
        let steal_addr = steal.local_addr().unwrap();
        let (png, sha) = sample_png();

        let bounce_handle = thread::spawn(move || {
            let (mut stream, _) = bounce.accept().unwrap();
            let buf = read_http_request(&mut stream);
            let split = buf
                .windows(4)
                .position(|w| w == b"\r\n\r\n")
                .expect("header/body split");
            let headers = String::from_utf8_lossy(&buf[..split]);
            assert!(
                headers.contains("X-Api-Key: leak-me"),
                "original hop must still send the key: {headers}"
            );
            let body = format!(
                "HTTP/1.1 302 Found\r\nLocation: http://{steal_addr}/stolen\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(body.as_bytes()).unwrap();
        });

        let (tx, rx) = mpsc::channel();
        let steal_handle = thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_millis(400);
            let mut contacted = false;
            while std::time::Instant::now() < deadline {
                match steal.accept() {
                    Ok((mut stream, _)) => {
                        let _ = read_http_request(&mut stream);
                        contacted = true;
                        break;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(err) => panic!("redirect target accept: {err}"),
                }
            }
            tx.send(contacted).unwrap();
        });

        let sink = HttpPostSink::new(
            format!("http://{bounce_addr}/ingest"),
            "X-Api-Key",
            Some("leak-me".into()),
        );
        let err = ingest_png(&png, rec(&sha), &sink, false).unwrap_err();
        match err {
            ZenclipError::Http(msg) => {
                assert!(
                    msg.contains("302") || msg.contains("redirect") || msg.contains("3"),
                    "expected 302 failure, got {msg}"
                );
            }
            other => panic!("unexpected {other}"),
        }
        bounce_handle.join().unwrap();
        steal_handle.join().unwrap();
        assert!(
            !rx.recv().unwrap(),
            "authenticated POST followed a redirect; API key would leak to the Location host"
        );
    }
}
