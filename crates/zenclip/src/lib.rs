//! zenClip — WYSIWYG media yoink.
//!
//! Grab the pixels that are on screen (or re-render an existing image),
//! emit a metadata-free sRGB PNG, name it by SHA256, hand it to a sink.

pub mod capture;
pub mod config;
pub mod error;
pub mod hash;
pub mod ingest;
pub mod pipeline;
pub mod pixels;
pub mod png_chunks;
pub mod record;
pub mod sanitize;
pub mod yoink;

pub use capture::{FrameSource, MockCapture, Region, WindowInfo, XcapCapture};
pub use config::Config;
pub use error::{Result, ZenclipError};
pub use ingest::{HttpPostSink, InboxFolderSink, IngestSink, NullSink};
pub use pipeline::process_frame;
pub use pixels::RgbaFrame;
pub use record::{ClipEvent, ClipRecord, Outcome, SourceKind};
pub use sanitize::{decode_to_frame, encode_clean_png, reencode_from_bytes};
