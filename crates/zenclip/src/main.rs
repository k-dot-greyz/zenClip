use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use zenclip::capture::{FrameSource, MockCapture, Region, XcapCapture};
use zenclip::config::Config;
use zenclip::error::{Result, ZenclipError};
use zenclip::ingest::{HttpPostSink, InboxFolderSink, IngestSink, NullSink};
use zenclip::pipeline::process_frame;
use zenclip::record::SourceKind;
use zenclip::yoink;

#[derive(Parser, Debug)]
#[command(
    name = "zenclip",
    version,
    about = "WYSIWYG media yoink: capture or re-render pixels, emit a metadata-free PNG."
)]
struct Cli {
    /// Path to zenclip.yaml. Defaults are used when omitted.
    #[arg(long, global = true, env = "ZENCLIP_CONFIG")]
    config: Option<PathBuf>,

    /// Override inbox directory.
    #[arg(long, global = true)]
    inbox: Option<PathBuf>,

    /// Sanitize and hash, print JSON, write nothing.
    #[arg(long, global = true)]
    dry_run: bool,

    /// Use an in-memory checkerboard instead of the real screen (CI / headless).
    #[arg(long, global = true)]
    mock: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Grab exactly what is on screen right now.
    Shot {
        /// Entire monitor.
        #[arg(long, group = "target")]
        full: bool,
        /// Crop of the monitor: x,y,w,h in pixels.
        #[arg(long, group = "target")]
        region: Option<String>,
        /// Window title substring or id.
        #[arg(long, group = "target")]
        window: Option<String>,
        /// Monitor index (0-based). Default: first.
        #[arg(long)]
        monitor: Option<usize>,
    },
    /// Re-render an existing image (file path, http(s) URL, or clipboard image).
    Yoink {
        /// File path or http(s) URL. Omit when using --clipboard.
        source: Option<String>,
        /// Yoink the image currently on the clipboard (pixels, not the file).
        #[arg(long)]
        clipboard: bool,
    },
    /// List capturable windows (live capture only).
    Windows,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("zenclip: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = Config::load(cli.config.as_deref())?;
    if let Some(inbox) = cli.inbox {
        cfg.inbox_dir = inbox;
    }

    match cli.command {
        Command::Windows => {
            let src = live_or_mock(cli.mock)?;
            for w in src.list_windows()? {
                println!("{}\t{}x{}\t{}", w.id, w.width, w.height, w.title);
            }
            return Ok(());
        }
        Command::Shot {
            full,
            region,
            window,
            monitor,
        } => {
            let src = live_or_mock(cli.mock)?;
            let (frame, kind) = if full || (region.is_none() && window.is_none()) {
                (src.capture_full(monitor)?, SourceKind::ShotFull)
            } else if let Some(spec) = region {
                let r = Region::parse(&spec)?;
                (src.capture_region(r, monitor)?, SourceKind::ShotRegion)
            } else if let Some(q) = window {
                (src.capture_window(&q)?, SourceKind::ShotWindow)
            } else {
                return Err(ZenclipError::InvalidFrame(
                    "specify --full, --region x,y,w,h, or --window <title>",
                ));
            };
            emit(&cfg, &*sink(&cfg, cli.dry_run)?, kind, &frame, cli.dry_run)?;
        }
        Command::Yoink { source, clipboard } => {
            let (frame, kind) = if clipboard {
                (yoink::yoink_clipboard()?, SourceKind::YoinkClipboard)
            } else if let Some(arg) = source {
                let kind = if arg.starts_with("http://") || arg.starts_with("https://") {
                    SourceKind::YoinkUrl
                } else {
                    SourceKind::YoinkFile
                };
                (yoink::yoink_arg(&arg, cfg.max_yoink_bytes)?, kind)
            } else {
                return Err(ZenclipError::InvalidFrame(
                    "yoink needs a file/url argument or --clipboard",
                ));
            };
            emit(&cfg, &*sink(&cfg, cli.dry_run)?, kind, &frame, cli.dry_run)?;
        }
    }
    Ok(())
}

fn live_or_mock(mock: bool) -> Result<Box<dyn FrameSource>> {
    if mock {
        Ok(Box::new(MockCapture::checkerboard(64, 48)))
    } else {
        Ok(Box::new(XcapCapture::new()))
    }
}

fn sink(cfg: &Config, dry_run: bool) -> Result<Box<dyn IngestSink>> {
    if dry_run {
        return Ok(Box::new(NullSink));
    }
    if cfg.http_post.enabled {
        if let Some(http) = HttpPostSink::from_config(cfg)? {
            return Ok(Box::new(http));
        }
    }
    Ok(Box::new(InboxFolderSink::from_config(cfg)?))
}

fn emit(
    cfg: &Config,
    sink: &dyn IngestSink,
    kind: SourceKind,
    frame: &zenclip::RgbaFrame,
    dry_run: bool,
) -> Result<()> {
    let event = process_frame(frame, kind, cfg, sink, dry_run)?;
    let json = serde_json::to_string(&event)?;
    let mut out = io::stdout().lock();
    writeln!(out, "{json}")?;
    Ok(())
}
