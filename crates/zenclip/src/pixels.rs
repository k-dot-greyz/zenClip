/// Raw 8-bit RGBA pixels. This is the only in-memory picture type zenClip
/// carries — no EXIF, no ICC, no timestamps, just what was visible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaFrame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl RgbaFrame {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> crate::error::Result<Self> {
        if width == 0 || height == 0 {
            return Err(crate::error::ZenclipError::EmptyFrame);
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or(crate::error::ZenclipError::InvalidFrame(
                "frame dimensions overflow",
            ))?;
        if pixels.len() != expected {
            return Err(crate::error::ZenclipError::InvalidFrame(
                "pixel buffer length does not match width*height*4",
            ));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }

    /// Crop to an inclusive-start, exclusive-end rectangle in pixel space.
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> crate::error::Result<Self> {
        if w == 0 || h == 0 {
            return Err(crate::error::ZenclipError::EmptyFrame);
        }
        if x.checked_add(w).is_none()
            || y.checked_add(h).is_none()
            || x + w > self.width
            || y + h > self.height
        {
            return Err(crate::error::ZenclipError::RegionOutOfBounds {
                region: format!("{x},{y},{w},{h}"),
                width: self.width,
                height: self.height,
            });
        }
        let mut out = Vec::with_capacity((w as usize) * (h as usize) * 4);
        for row in y..(y + h) {
            let start = ((row * self.width + x) as usize) * 4;
            let end = start + (w as usize) * 4;
            out.extend_from_slice(&self.pixels[start..end]);
        }
        Self::new(w, h, out)
    }
}

#[cfg(test)]
mod tests {
    use super::RgbaFrame;

    #[test]
    fn rejects_empty_and_mismatched_buffers() {
        assert!(RgbaFrame::new(0, 1, vec![0; 4]).is_err());
        assert!(RgbaFrame::new(1, 1, vec![0; 3]).is_err());
    }

    #[test]
    fn crop_copies_the_requested_pixels_only() {
        // 2x2: R G / B W
        let mut px = vec![0u8; 16];
        px[0..4].copy_from_slice(&[255, 0, 0, 255]);
        px[4..8].copy_from_slice(&[0, 255, 0, 255]);
        px[8..12].copy_from_slice(&[0, 0, 255, 255]);
        px[12..16].copy_from_slice(&[255, 255, 255, 255]);
        let frame = RgbaFrame::new(2, 2, px).unwrap();
        let crop = frame.crop(1, 0, 1, 1).unwrap();
        assert_eq!(crop.width(), 1);
        assert_eq!(crop.height(), 1);
        assert_eq!(crop.pixels(), &[0, 255, 0, 255]);
        assert!(frame.crop(1, 0, 2, 1).is_err());
    }
}
