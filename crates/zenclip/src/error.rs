use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ZenclipError {
    #[error("empty frame: nothing to yoink")]
    EmptyFrame,

    #[error("invalid frame: {0}")]
    InvalidFrame(&'static str),

    #[error("invalid image ({format}): {reason}")]
    InvalidImage {
        format: &'static str,
        reason: String,
    },

    #[error("region {region} is outside the {width}x{height} frame")]
    RegionOutOfBounds {
        region: String,
        width: u32,
        height: u32,
    },

    #[error("capture failed: {0}")]
    Capture(String),

    #[error("clipboard: {0}")]
    Clipboard(String),

    #[error("window not found: {0}")]
    WindowNotFound(String),

    #[error("ambiguous window query '{query}', matches: {titles:?}")]
    AmbiguousWindow { query: String, titles: Vec<String> },

    #[error("unsupported URL scheme '{scheme}' (http/https only)")]
    UnsupportedUrlScheme { scheme: String },

    #[error("yoink payload {size} bytes exceeds max {max}")]
    Oversize { size: u64, max: u64 },

    #[error("http yoink failed: {0}")]
    Http(String),

    #[error("png encode failed: {0}")]
    Png(String),

    #[error("config error in {path}: {reason}", path = path.display())]
    Config { path: PathBuf, reason: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),

    #[error("live capture support was not compiled in (rebuild with --features live-capture)")]
    LiveCaptureDisabled,

    #[error("clipboard support was not compiled in (rebuild with --features clipboard)")]
    ClipboardDisabled,

    #[error("http support was not compiled in (rebuild with --features http)")]
    HttpDisabled,
}

pub type Result<T> = std::result::Result<T, ZenclipError>;
