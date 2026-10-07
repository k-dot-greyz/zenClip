use std::fs;
use std::io::Read;
use std::path::Path;

use url::Url;

use crate::error::{Result, ZenclipError};
use crate::pixels::RgbaFrame;
use crate::sanitize::decode_to_frame;

/// Load pixels from a file, http(s) URL, or the clipboard image buffer.
///
/// Whatever the container was (JPEG with GPS, PNG with tEXt, …), we decode
/// to samples and throw the container away. That is the yoink: re-render
/// what was visible, no more, no less.
pub fn yoink_file(path: &Path) -> Result<RgbaFrame> {
    let bytes = fs::read(path)?;
    decode_to_frame(&bytes)
}

pub fn yoink_url(url: &str, max_bytes: u64) -> Result<RgbaFrame> {
    let parsed = Url::parse(url).map_err(|e| ZenclipError::InvalidImage {
        format: "url",
        reason: e.to_string(),
    })?;
    match parsed.scheme() {
        "http" | "https" => {}
        other => {
            return Err(ZenclipError::UnsupportedUrlScheme {
                scheme: other.into(),
            });
        }
    }
    #[cfg(feature = "http")]
    {
        let resp = ureq::get(url)
            .call()
            .map_err(|e| ZenclipError::Http(e.to_string()))?;
        if let Some(len) = resp.header("Content-Length") {
            if let Ok(n) = len.parse::<u64>() {
                if n > max_bytes {
                    return Err(ZenclipError::Oversize {
                        size: n,
                        max: max_bytes,
                    });
                }
            }
        }
        let mut reader = resp.into_reader().take(max_bytes.saturating_add(1));
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        if buf.len() as u64 > max_bytes {
            return Err(ZenclipError::Oversize {
                size: buf.len() as u64,
                max: max_bytes,
            });
        }
        decode_to_frame(&buf)
    }
    #[cfg(not(feature = "http"))]
    {
        let _ = max_bytes;
        Err(ZenclipError::HttpDisabled)
    }
}

pub fn yoink_clipboard() -> Result<RgbaFrame> {
    #[cfg(feature = "clipboard")]
    {
        let mut cb =
            arboard::Clipboard::new().map_err(|e| ZenclipError::Clipboard(e.to_string()))?;
        match cb.get_image() {
            Ok(img) => {
                let w = u32::try_from(img.width).map_err(|_| ZenclipError::EmptyFrame)?;
                let h = u32::try_from(img.height).map_err(|_| ZenclipError::EmptyFrame)?;
                // arboard documents RGBA8, 4 bytes/pixel, unpremultiplied.
                RgbaFrame::new(w, h, img.bytes.into_owned())
            }
            Err(err) => Err(ZenclipError::Clipboard(err.to_string())),
        }
    }
    #[cfg(not(feature = "clipboard"))]
    {
        Err(ZenclipError::ClipboardDisabled)
    }
}

pub fn yoink_arg(arg: &str, max_bytes: u64) -> Result<RgbaFrame> {
    if arg.starts_with("http://") || arg.starts_with("https://") {
        yoink_url(arg, max_bytes)
    } else {
        yoink_file(Path::new(arg))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sanitize::encode_clean_png;
    use std::io::Write;

    #[test]
    fn yoink_file_roundtrip_pixels() {
        let dir = tempfile::tempdir().unwrap();
        let frame = RgbaFrame::new(2, 1, vec![9, 8, 7, 255, 1, 2, 3, 255]).unwrap();
        let png = encode_clean_png(&frame).unwrap();
        let path = dir.path().join("a.png");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(&png).unwrap();
        let got = yoink_file(&path).unwrap();
        assert_eq!(got, frame);
    }

    #[test]
    fn rejects_file_url_scheme() {
        let err = yoink_url("file:///etc/passwd", 100).unwrap_err();
        match err {
            ZenclipError::UnsupportedUrlScheme { scheme } => assert_eq!(scheme, "file"),
            other => panic!("unexpected {other}"),
        }
    }
}
