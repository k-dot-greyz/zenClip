use crate::config::Config;
use crate::error::Result;
use crate::hash::sha256_hex;
use crate::ingest::{ingest_png, IngestSink};
use crate::pixels::RgbaFrame;
use crate::record::{ClipEvent, ClipRecord, SourceKind};
use crate::sanitize::encode_clean_png;

/// Capture/yoink pixels → metadata-free PNG → SHA256 name → sink.
pub fn process_frame(
    frame: &RgbaFrame,
    source: SourceKind,
    cfg: &Config,
    sink: &dyn IngestSink,
    dry_run: bool,
) -> Result<ClipEvent> {
    let png = encode_clean_png(frame)?;
    let sha = sha256_hex(&png);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .ok();
    let record = ClipRecord {
        sha256: sha,
        width: cfg.sidecar.dimensions.then_some(frame.width()),
        height: cfg.sidecar.dimensions.then_some(frame.height()),
        source: cfg.sidecar.source_kind.then_some(source),
        captured_at: if cfg.sidecar.capture_time { now } else { None },
    };
    ingest_png(&png, record, sink, dry_run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::{FrameSource, MockCapture, Region};
    use crate::ingest::InboxFolderSink;

    #[test]
    fn mock_shot_lands_in_inbox_as_clean_hash_named_png() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = Config {
            inbox_dir: dir.path().to_path_buf(),
            ..Config::default()
        };
        let sink = InboxFolderSink::from_config(&cfg).unwrap();
        let src = MockCapture::checkerboard(16, 16);
        let frame = src.capture_full(None).unwrap();
        let event = process_frame(&frame, SourceKind::ShotFull, &cfg, &sink, false).unwrap();
        assert_eq!(event.outcome, crate::record::Outcome::Completed);
        let path = event.path.expect("path");
        let bytes = std::fs::read(&path).unwrap();
        crate::png_chunks::assert_clean_png(&bytes).unwrap();
        assert!(
            path.ends_with(&format!("{}.png", event.sha256)),
            "expected hash-named png, got {path}"
        );
        // same pixels again → dedup
        let again = process_frame(&frame, SourceKind::ShotFull, &cfg, &sink, false).unwrap();
        assert_eq!(again.outcome, crate::record::Outcome::Duplicate);
    }

    #[test]
    fn region_shot_is_smaller_than_full() {
        let src = MockCapture::checkerboard(32, 16);
        let full = src.capture_full(None).unwrap();
        let region = src
            .capture_region(Region::parse("0,0,8,8").unwrap(), None)
            .unwrap();
        assert!(region.pixels().len() < full.pixels().len());
        assert_eq!(region.width(), 8);
    }
}
