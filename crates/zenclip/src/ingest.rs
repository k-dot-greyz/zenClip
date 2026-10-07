use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{Result, ZenclipError};
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

    pub fn png_path(&self, sha256: &str) -> PathBuf {
        self.dir.join(format!("{sha256}.png"))
    }

    pub fn sidecar_path(&self, sha256: &str) -> PathBuf {
        self.dir.join(format!("{sha256}.json"))
    }
}

impl IngestSink for InboxFolderSink {
    fn contains(&self, sha256: &str) -> Result<bool> {
        Ok(self.png_path(sha256).exists())
    }

    fn put(&self, png: &[u8], record: &ClipRecord) -> Result<PutResult> {
        let dest = self.png_path(&record.sha256);
        if dest.exists() {
            return Ok(PutResult { path: Some(dest) });
        }
        let tmp = dest.with_extension("png.partial");
        fs::write(&tmp, png)?;
        fs::rename(&tmp, &dest)?;
        if self.write_sidecar {
            let side = self.sidecar_path(&record.sha256);
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
            let mut req = ureq::post(&self.url)
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
        let dest = sink.png_path(&sha);
        assert!(dest.exists());
        assert_eq!(fs::read(&dest).unwrap(), png);
        let side: ClipRecord =
            serde_json::from_slice(&fs::read(sink.sidecar_path(&sha)).unwrap()).unwrap();
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
        assert!(sink.png_path(&sha).exists());
        assert!(!sink.sidecar_path(&sha).exists());
    }

    #[test]
    fn dry_run_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let sink = InboxFolderSink::new(dir.path(), true).unwrap();
        let (png, sha) = sample_png();
        let event = ingest_png(&png, rec(&sha), &sink, true).unwrap();
        assert_eq!(event.outcome, Outcome::DryRun);
        assert!(!sink.png_path(&sha).exists());
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
        assert!(sink.png_path(&sha_a).exists());
        assert!(sink.png_path(&sha_b).exists());
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
}
