use crate::error::{Result, ZenclipError};
use crate::pixels::RgbaFrame;

/// Rectangle in pixel space: `x,y,w,h`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn parse(spec: &str) -> Result<Self> {
        let parts: Vec<_> = spec.split(',').map(str::trim).collect();
        if parts.len() != 4 {
            return Err(ZenclipError::InvalidFrame(
                "region must be x,y,w,h (four comma-separated integers)",
            ));
        }
        let parse = |s: &str| {
            s.parse::<u32>()
                .map_err(|_| ZenclipError::InvalidFrame("region values must be u32"))
        };
        Ok(Self {
            x: parse(parts[0])?,
            y: parse(parts[1])?,
            width: parse(parts[2])?,
            height: parse(parts[3])?,
        })
    }

    pub fn as_tuple(&self) -> (u32, u32, u32, u32) {
        (self.x, self.y, self.width, self.height)
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{},{}", self.x, self.y, self.width, self.height)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
}

/// Grab pixels that are on screen *right now*. Implementations must return
/// raw RGBA, not an encoded screenshot with metadata.
pub trait FrameSource {
    fn capture_full(&self, monitor: Option<usize>) -> Result<RgbaFrame>;
    fn capture_region(&self, region: Region, monitor: Option<usize>) -> Result<RgbaFrame>;
    fn capture_window(&self, query: &str) -> Result<RgbaFrame>;
    fn list_windows(&self) -> Result<Vec<WindowInfo>>;
}

/// In-memory capture used by tests and `--fixture` replay. CI is headless.
#[derive(Clone, Debug)]
pub struct MockCapture {
    frame: RgbaFrame,
    windows: Vec<WindowInfo>,
}

impl MockCapture {
    pub fn new(frame: RgbaFrame) -> Self {
        Self {
            windows: vec![WindowInfo {
                id: "1".into(),
                title: "mock-window".into(),
                width: frame.width(),
                height: frame.height(),
            }],
            frame,
        }
    }

    pub fn with_windows(mut self, windows: Vec<WindowInfo>) -> Self {
        self.windows = windows;
        self
    }

    pub fn checkerboard(width: u32, height: u32) -> Self {
        let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        for y in 0..height {
            for x in 0..width {
                let i = ((y * width + x) as usize) * 4;
                let on = ((x / 8) + (y / 8)) % 2 == 0;
                let v = if on { 240 } else { 32 };
                pixels[i] = v;
                pixels[i + 1] = v;
                pixels[i + 2] = if on { 240 } else { 48 };
                pixels[i + 3] = 255;
            }
        }
        Self::new(RgbaFrame::new(width, height, pixels).expect("checkerboard"))
    }
}

impl FrameSource for MockCapture {
    fn capture_full(&self, _monitor: Option<usize>) -> Result<RgbaFrame> {
        Ok(self.frame.clone())
    }

    fn capture_region(&self, region: Region, _monitor: Option<usize>) -> Result<RgbaFrame> {
        self.frame
            .crop(region.x, region.y, region.width, region.height)
    }

    fn capture_window(&self, query: &str) -> Result<RgbaFrame> {
        pick_window(&self.windows, query)?;
        Ok(self.frame.clone())
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        Ok(self.windows.clone())
    }
}

pub(crate) fn pick_window<'a>(windows: &'a [WindowInfo], query: &str) -> Result<&'a WindowInfo> {
    let q = query.to_lowercase();
    let matches: Vec<_> = windows
        .iter()
        .filter(|w| w.title.to_lowercase().contains(&q) || w.id == query)
        .collect();
    match matches.len() {
        0 => Err(ZenclipError::WindowNotFound(query.into())),
        1 => Ok(matches[0]),
        _ => Err(ZenclipError::AmbiguousWindow {
            query: query.into(),
            titles: matches.iter().map(|w| w.title.clone()).collect(),
        }),
    }
}

/// Live capture via [`xcap`](https://crates.io/crates/xcap).
///
/// Chosen over `screenshots` because xcap is the maintained successor, returns
/// `RgbaImage` (pixels, not an encoded file), and covers Windows / macOS /
/// Linux X11. Wayland is a known caveat: xcap marks screen/window capture as
/// incomplete there (portal / protocol limits).
#[cfg(feature = "live-capture")]
pub struct XcapCapture;

#[cfg(feature = "live-capture")]
impl XcapCapture {
    pub fn new() -> Self {
        Self
    }

    fn monitor(monitor: Option<usize>) -> Result<xcap::Monitor> {
        let monitors = xcap::Monitor::all().map_err(|e| ZenclipError::Capture(e.to_string()))?;
        if monitors.is_empty() {
            return Err(ZenclipError::Capture("no monitors".into()));
        }
        match monitor {
            None => Ok(monitors.into_iter().next().unwrap()),
            Some(i) => monitors
                .into_iter()
                .nth(i)
                .ok_or_else(|| ZenclipError::Capture(format!("monitor index {i} out of range"))),
        }
    }

    fn rgba_from_xcap(img: xcap::image::RgbaImage) -> Result<RgbaFrame> {
        let (w, h) = img.dimensions();
        RgbaFrame::new(w, h, img.into_raw())
    }
}

#[cfg(feature = "live-capture")]
impl Default for XcapCapture {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "live-capture")]
impl FrameSource for XcapCapture {
    fn capture_full(&self, monitor: Option<usize>) -> Result<RgbaFrame> {
        let mon = Self::monitor(monitor)?;
        let img = mon
            .capture_image()
            .map_err(|e| ZenclipError::Capture(e.to_string()))?;
        Self::rgba_from_xcap(img)
    }

    fn capture_region(&self, region: Region, monitor: Option<usize>) -> Result<RgbaFrame> {
        let full = self.capture_full(monitor)?;
        full.crop(region.x, region.y, region.width, region.height)
    }

    fn capture_window(&self, query: &str) -> Result<RgbaFrame> {
        let listed = self.list_windows()?;
        let info = pick_window(&listed, query)?;
        let windows = xcap::Window::all().map_err(|e| ZenclipError::Capture(e.to_string()))?;
        let win = windows
            .into_iter()
            .find(|w| {
                let id_ok = w.id().map(|id| id.to_string() == info.id).unwrap_or(false);
                let title_ok = w.title().map(|t| t == info.title).unwrap_or(false);
                id_ok || title_ok
            })
            .ok_or_else(|| ZenclipError::WindowNotFound(query.into()))?;
        if win.is_minimized().unwrap_or(false) {
            let title = win.title().unwrap_or_else(|_| query.to_string());
            return Err(ZenclipError::Capture(format!(
                "window '{title}' is minimized"
            )));
        }
        let img = win
            .capture_image()
            .map_err(|e| ZenclipError::Capture(e.to_string()))?;
        Self::rgba_from_xcap(img)
    }

    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        let windows = xcap::Window::all().map_err(|e| ZenclipError::Capture(e.to_string()))?;
        let mut out = Vec::new();
        for w in windows {
            let id = w.id().map_err(|e| ZenclipError::Capture(e.to_string()))?;
            let title = w
                .title()
                .map_err(|e| ZenclipError::Capture(e.to_string()))?;
            let width = w
                .width()
                .map_err(|e| ZenclipError::Capture(e.to_string()))?;
            let height = w
                .height()
                .map_err(|e| ZenclipError::Capture(e.to_string()))?;
            out.push(WindowInfo {
                id: id.to_string(),
                title,
                width,
                height,
            });
        }
        Ok(out)
    }
}

#[cfg(not(feature = "live-capture"))]
pub struct XcapCapture;

#[cfg(not(feature = "live-capture"))]
impl XcapCapture {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "live-capture"))]
impl FrameSource for XcapCapture {
    fn capture_full(&self, _: Option<usize>) -> Result<RgbaFrame> {
        Err(ZenclipError::LiveCaptureDisabled)
    }
    fn capture_region(&self, _: Region, _: Option<usize>) -> Result<RgbaFrame> {
        Err(ZenclipError::LiveCaptureDisabled)
    }
    fn capture_window(&self, _: &str) -> Result<RgbaFrame> {
        Err(ZenclipError::LiveCaptureDisabled)
    }
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        Err(ZenclipError::LiveCaptureDisabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_region_is_exact_crop() {
        let src = MockCapture::checkerboard(32, 32);
        let region = Region::parse("8,8,8,8").unwrap();
        let crop = src.capture_region(region, None).unwrap();
        assert_eq!(crop.width(), 8);
        assert_eq!(crop.height(), 8);
    }

    #[test]
    fn mock_window_query() {
        let src = MockCapture::checkerboard(4, 4);
        assert!(src.capture_window("mock").is_ok());
        assert!(src.capture_window("nope").is_err());
    }

    #[test]
    fn region_parse_rejects_junk() {
        assert!(Region::parse("1,2,3").is_err());
        assert!(Region::parse("a,b,c,d").is_err());
        assert_eq!(Region::parse("1,2,3,4").unwrap().as_tuple(), (1, 2, 3, 4));
    }
}
