use serde::{Deserialize, Serialize};

/// Kind of capture that produced the pixels. Stored in the sidecar, never in the PNG.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    ShotFull,
    ShotRegion,
    ShotWindow,
    YoinkFile,
    YoinkUrl,
    YoinkClipboard,
    Replay,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ShotFull => "shot.full",
            Self::ShotRegion => "shot.region",
            Self::ShotWindow => "shot.window",
            Self::YoinkFile => "yoink.file",
            Self::YoinkUrl => "yoink.url",
            Self::YoinkClipboard => "yoink.clipboard",
            Self::Replay => "replay",
        }
    }
}

/// Pipeline record. Lives next to the PNG (sidecar) or in a sink's own store.
/// Never written into PNG chunks.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipRecord {
    pub sha256: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceKind>,
    /// Unix seconds. Optional so a paranoid config can omit time entirely.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Completed,
    Duplicate,
    DryRun,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipEvent {
    pub stage: &'static str,
    pub sha256: String,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}
