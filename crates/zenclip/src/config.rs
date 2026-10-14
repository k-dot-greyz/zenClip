use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, ZenclipError};

/// Runtime config. Nothing about capture/ingest is hardcoded in domain logic.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Where the default folder sink writes `{sha256}.png`.
    #[serde(default = "default_inbox")]
    pub inbox_dir: PathBuf,
    /// Write `{sha256}.json` next to the PNG. The PNG itself never gets this.
    #[serde(default = "default_true")]
    pub write_sidecar: bool,
    #[serde(default)]
    pub sidecar: SidecarConfig,
    #[serde(default)]
    pub http_post: HttpPostConfig,
    /// Max bytes fetched for `yoink <url>`.
    #[serde(default = "default_max_yoink")]
    pub max_yoink_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidecarConfig {
    #[serde(default = "default_true")]
    pub capture_time: bool,
    #[serde(default = "default_true")]
    pub source_kind: bool,
    #[serde(default = "default_true")]
    pub dimensions: bool,
}

impl Default for SidecarConfig {
    fn default() -> Self {
        Self {
            capture_time: true,
            source_kind: true,
            dimensions: true,
        }
    }
}

/// Optional generic HTTP POST sink. This is **not** the zenOS inbox contract
/// (unknown here). See README "Ingest sinks".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpPostConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: Option<String>,
    /// Header that carries a shared secret, if any. Value comes from env.
    #[serde(default = "default_api_header")]
    pub api_key_header: String,
}

impl Default for HttpPostConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: None,
            api_key_header: default_api_header(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            inbox_dir: default_inbox(),
            write_sidecar: true,
            sidecar: SidecarConfig::default(),
            http_post: HttpPostConfig::default(),
            max_yoink_bytes: default_max_yoink(),
        }
    }
}

fn default_inbox() -> PathBuf {
    PathBuf::from("inbox")
}
fn default_true() -> bool {
    true
}
fn default_max_yoink() -> u64 {
    20 * 1024 * 1024
}
fn default_api_header() -> String {
    "X-Api-Key".into()
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default().with_env_overrides());
        };
        if !path.exists() {
            return Err(ZenclipError::Config {
                path: path.to_path_buf(),
                reason: "file not found".into(),
            });
        }
        let raw = fs::read_to_string(path)?;
        let mut cfg: Config = serde_yaml::from_str(&raw).map_err(|e| ZenclipError::Config {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        cfg = cfg.with_env_overrides();
        Ok(cfg)
    }

    pub fn with_env_overrides(mut self) -> Self {
        if let Ok(dir) = std::env::var("ZENCLIP_INBOX_DIR") {
            if !dir.is_empty() {
                self.inbox_dir = PathBuf::from(dir);
            }
        }
        if let Ok(url) = std::env::var("ZENCLIP_HTTP_SINK_URL") {
            if !url.is_empty() {
                self.http_post.enabled = true;
                self.http_post.url = Some(url);
            }
        }
        if let Ok(max) = std::env::var("ZENCLIP_MAX_YOINK_BYTES") {
            if let Ok(n) = max.parse() {
                self.max_yoink_bytes = n;
            }
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn missing_file_fails_fast() {
        let err = Config::load(Some(Path::new("/no/such/zenclip.yaml"))).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not found"), "{msg}");
    }

    #[test]
    fn yaml_roundtrip_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zenclip.yaml");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(
            f,
            "inbox_dir: /tmp/zenclip-inbox\nwrite_sidecar: false\nmax_yoink_bytes: 12\n"
        )
        .unwrap();
        let cfg = Config::load(Some(&path)).unwrap();
        assert_eq!(cfg.inbox_dir, PathBuf::from("/tmp/zenclip-inbox"));
        assert!(!cfg.write_sidecar);
        assert_eq!(cfg.max_yoink_bytes, 12);
        assert!(!cfg.http_post.enabled);
    }
}
