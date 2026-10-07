use std::io::Cursor;

use image::{ImageDecoder, ImageReader};
use png::{BitDepth, ColorType, Encoder, SrgbRenderingIntent};

use crate::error::{Result, ZenclipError};
use crate::pixels::RgbaFrame;
use crate::png_chunks::assert_clean_png;

/// Decode any supported raster into raw RGBA pixels.
///
/// EXIF / XMP / ICC / comments are dropped because we never copy encoded
/// bytes — we only keep the sample values the decoder produced. EXIF
/// orientation is applied so the pixels match how a typical viewer shows
/// the file (WYSIWYG of "what it looked like"), then the orientation tag
/// itself is discarded with the rest of the metadata.
pub fn decode_to_frame(encoded: &[u8]) -> Result<RgbaFrame> {
    if encoded.is_empty() {
        return Err(ZenclipError::EmptyFrame);
    }
    let reader = ImageReader::new(Cursor::new(encoded))
        .with_guessed_format()
        .map_err(|e| ZenclipError::InvalidImage {
            format: "unknown",
            reason: e.to_string(),
        })?;
    let format = reader.format();
    let mut decoder = reader
        .into_decoder()
        .map_err(|e| ZenclipError::InvalidImage {
            format: format_name(format),
            reason: e.to_string(),
        })?;
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut dyn_img =
        image::DynamicImage::from_decoder(decoder).map_err(|e| ZenclipError::InvalidImage {
            format: format_name(format),
            reason: e.to_string(),
        })?;
    dyn_img.apply_orientation(orientation);
    let rgba = dyn_img.to_rgba8();
    let (w, h) = rgba.dimensions();
    RgbaFrame::new(w, h, rgba.into_raw())
}

fn format_name(format: Option<image::ImageFormat>) -> &'static str {
    match format {
        Some(image::ImageFormat::Jpeg) => "jpeg",
        Some(image::ImageFormat::Png) => "png",
        Some(image::ImageFormat::Gif) => "gif",
        Some(image::ImageFormat::WebP) => "webp",
        Some(image::ImageFormat::Bmp) => "bmp",
        _ => "unknown",
    }
}

/// Re-encode pixels as a metadata-free 8-bit RGBA PNG tagged sRGB.
///
/// Chunks are written from raw samples. We never copy tEXt/iTXt/zTXt,
/// tIME, eXIf, iCCP, pHYs, or any other ancillary metadata from a source
/// file. Colour is declared sRGB so the picture still looks the same in
/// colour-managed viewers.
pub fn encode_clean_png(frame: &RgbaFrame) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut encoder = Encoder::new(&mut out, frame.width(), frame.height());
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        encoder.set_source_srgb(SrgbRenderingIntent::Perceptual);
        let mut writer = encoder
            .write_header()
            .map_err(|e| ZenclipError::Png(e.to_string()))?;
        writer
            .write_image_data(frame.pixels())
            .map_err(|e| ZenclipError::Png(e.to_string()))?;
        writer
            .finish()
            .map_err(|e| ZenclipError::Png(e.to_string()))?;
    }
    assert_clean_png(&out)?;
    Ok(out)
}

/// Decode → pixels → clean PNG. Used by `yoink` for files, URLs, and
/// encoded clipboard payloads.
pub fn reencode_from_bytes(encoded: &[u8]) -> Result<Vec<u8>> {
    let frame = decode_to_frame(encoded)?;
    encode_clean_png(&frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::png_chunks::{chunk_types, parse_chunks, FORBIDDEN_CHUNKS};

    const GPS_CANARY: &str = "SECRET_GPS_CANARY_56N_24E";
    const TEXT_CANARY: &str = "SECRET_TEXT_CANARY";
    const ZTXT_CANARY: &str = "SECRET_ZTXT_CANARY";
    const ITXT_CANARY: &str = "SECRET_ITXT_CANARY";

    fn solid_frame(r: u8, g: u8, b: u8) -> RgbaFrame {
        let mut px = vec![0u8; 8 * 8 * 4];
        for pixel in px.as_chunks_mut::<4>().0 {
            pixel.copy_from_slice(&[r, g, b, 255]);
        }
        RgbaFrame::new(8, 8, px).unwrap()
    }

    fn jpeg_with_exif_and_gps() -> Vec<u8> {
        let frame = solid_frame(220, 20, 60);
        let rgb = image::RgbImage::from_fn(frame.width(), frame.height(), |x, y| {
            let i = ((y * frame.width() + x) as usize) * 4;
            image::Rgb([
                frame.pixels()[i],
                frame.pixels()[i + 1],
                frame.pixels()[i + 2],
            ])
        });
        let mut jpeg = Vec::new();
        rgb.write_to(&mut Cursor::new(&mut jpeg), image::ImageFormat::Jpeg)
            .expect("jpeg encode");
        inject_exif_app1(&mut jpeg, GPS_CANARY.as_bytes());
        jpeg
    }

    /// Insert a little-endian TIFF/EXIF APP1 with ImageDescription + GPS IFD
    /// immediately after SOI. Length-prefixed JPEG segment.
    fn inject_exif_app1(jpeg: &mut Vec<u8>, description: &[u8]) {
        assert_eq!(&jpeg[..2], &[0xFF, 0xD8], "SOI");
        let app1 = build_exif_payload(description);
        let mut segment = Vec::with_capacity(4 + app1.len());
        segment.extend_from_slice(&[0xFF, 0xE1]);
        let len = u16::try_from(app1.len() + 2).expect("APP1 too large");
        segment.extend_from_slice(&len.to_be_bytes());
        segment.extend_from_slice(&app1);
        jpeg.splice(2..2, segment);
    }

    fn build_exif_payload(description: &[u8]) -> Vec<u8> {
        // "Exif\0\0" + TIFF. Offsets below are TIFF-relative (after the 6-byte header).
        let mut desc = description.to_vec();
        if desc.last() != Some(&0) {
            desc.push(0);
        }
        let desc_len = desc.len() as u32;

        // Layout:
        //  0: II* + IFD0 offset 8
        //  8: IFD0 (2 entries) + next=0          = 2+24+4 = 30 bytes  → 8..38
        // 38: ImageDescription ASCII
        // gps_ifd at 38+desc_len, aligned
        let desc_off = 38u32;
        let gps_ifd = align4(desc_off + desc_len);
        // GPS IFD: 4 entries + next=0 = 2+48+4 = 54, then 6 rationals (48 bytes)
        let gps_lat_off = gps_ifd + 54;
        let gps_lon_off = gps_lat_off + 24;

        let mut tiff = vec![0u8; gps_lon_off as usize + 24];
        tiff[0..4].copy_from_slice(&[b'I', b'I', 0x2A, 0x00]);
        tiff[4..8].copy_from_slice(&8u32.to_le_bytes());

        // IFD0 entry count
        tiff[8..10].copy_from_slice(&2u16.to_le_bytes());
        write_ifd_entry(&mut tiff, 10, 0x010E, 2, desc_len, desc_off); // ImageDescription ASCII
        write_ifd_entry(&mut tiff, 22, 0x8825, 4, 1, gps_ifd); // GPSInfo LONG
        tiff[34..38].copy_from_slice(&0u32.to_le_bytes());
        tiff[desc_off as usize..desc_off as usize + desc.len()].copy_from_slice(&desc);

        let gps = gps_ifd as usize;
        tiff[gps..gps + 2].copy_from_slice(&4u16.to_le_bytes());
        // GPSLatitudeRef ASCII "N\0" inlined
        write_ifd_entry(
            &mut tiff,
            gps + 2,
            0x0001,
            2,
            2,
            u32::from_le_bytes(*b"N\0\0\0"),
        );
        write_ifd_entry(&mut tiff, gps + 14, 0x0002, 5, 3, gps_lat_off); // GPSLatitude RATIONAL×3
        write_ifd_entry(
            &mut tiff,
            gps + 26,
            0x0003,
            2,
            2,
            u32::from_le_bytes(*b"E\0\0\0"),
        );
        write_ifd_entry(&mut tiff, gps + 38, 0x0004, 5, 3, gps_lon_off);
        tiff[gps + 50..gps + 54].copy_from_slice(&0u32.to_le_bytes());

        // 56° N, 24° E (Riga-ish, integer degrees)
        write_rationals(&mut tiff, gps_lat_off as usize, &[(56, 1), (0, 1), (0, 1)]);
        write_rationals(&mut tiff, gps_lon_off as usize, &[(24, 1), (0, 1), (0, 1)]);

        let mut app1 = b"Exif\0\0".to_vec();
        app1.extend_from_slice(&tiff);
        app1
    }

    fn align4(n: u32) -> u32 {
        (n + 3) & !3
    }

    fn write_ifd_entry(buf: &mut [u8], at: usize, tag: u16, typ: u16, count: u32, value: u32) {
        buf[at..at + 2].copy_from_slice(&tag.to_le_bytes());
        buf[at + 2..at + 4].copy_from_slice(&typ.to_le_bytes());
        buf[at + 4..at + 8].copy_from_slice(&count.to_le_bytes());
        buf[at + 8..at + 12].copy_from_slice(&value.to_le_bytes());
    }

    fn write_rationals(buf: &mut [u8], at: usize, vals: &[(u32, u32)]) {
        for (i, (num, den)) in vals.iter().enumerate() {
            let o = at + i * 8;
            buf[o..o + 4].copy_from_slice(&num.to_le_bytes());
            buf[o + 4..o + 8].copy_from_slice(&den.to_le_bytes());
        }
    }

    fn png_with_text_chunks() -> Vec<u8> {
        let frame = solid_frame(30, 90, 200);
        let mut out = Vec::new();
        {
            let mut encoder = Encoder::new(&mut out, frame.width(), frame.height());
            encoder.set_color(ColorType::Rgba);
            encoder.set_depth(BitDepth::Eight);
            encoder
                .add_text_chunk("Comment".into(), TEXT_CANARY.into())
                .unwrap();
            encoder
                .add_ztxt_chunk("Software".into(), ZTXT_CANARY.into())
                .unwrap();
            encoder
                .add_itxt_chunk("Author".into(), ITXT_CANARY.into())
                .unwrap();
            let mut writer = encoder.write_header().unwrap();
            // tIME: year/month/day/hour/min/sec as a raw ancillary chunk
            writer
                .write_chunk(
                    png::chunk::ChunkType(*b"tIME"),
                    &[0x07, 0xEA, 0x0A, 0x07, 0x0C, 0x00, 0x00],
                )
                .unwrap();
            writer.write_image_data(frame.pixels()).unwrap();
            writer.finish().unwrap();
        }
        out
    }

    fn jpeg_has_exif_and_gps(jpeg: &[u8]) -> bool {
        jpeg.windows(4).any(|w| w == b"Exif")
            && jpeg
                .windows(GPS_CANARY.len())
                .any(|w| w == GPS_CANARY.as_bytes())
            && jpeg.windows(2).any(|w| w == [0x25, 0x88]) // GPS IFD pointer tag 0x8825 LE
    }

    fn png_has_text_canaries(png: &[u8]) -> bool {
        let types = chunk_types(png).unwrap();
        let has = |name: &[u8; 4]| types.iter().any(|t| t == name);
        // zTXt payload is zlib-compressed, so the canary is not searchable as raw bytes.
        has(b"tEXt")
            && has(b"zTXt")
            && has(b"iTXt")
            && has(b"tIME")
            && png
                .windows(TEXT_CANARY.len())
                .any(|w| w == TEXT_CANARY.as_bytes())
            && png
                .windows(ITXT_CANARY.len())
                .any(|w| w == ITXT_CANARY.as_bytes())
    }

    #[test]
    fn jpeg_fixture_actually_contains_exif_and_gps() {
        let jpeg = jpeg_with_exif_and_gps();
        assert!(
            jpeg_has_exif_and_gps(&jpeg),
            "fixture must carry EXIF+GPS before sanitize"
        );
    }

    #[test]
    fn png_fixture_actually_contains_text_and_time_chunks() {
        let png = png_with_text_chunks();
        let types = chunk_types(&png)
            .unwrap()
            .into_iter()
            .map(|t| String::from_utf8_lossy(&t).into_owned())
            .collect::<Vec<_>>();
        assert!(
            png_has_text_canaries(&png),
            "fixture must carry text/time chunks before sanitize, got {types:?}"
        );
    }

    #[test]
    fn jpeg_exif_gps_stripped_on_reencode() {
        let jpeg = jpeg_with_exif_and_gps();
        let out = reencode_from_bytes(&jpeg).expect("sanitize jpeg");
        crate::png_chunks::assert_clean_png(&out).unwrap();
        let types = chunk_types(&out).unwrap();
        for forbidden in FORBIDDEN_CHUNKS {
            assert!(
                !types.iter().any(|t| t == *forbidden),
                "forbidden chunk {} survived",
                String::from_utf8_lossy(*forbidden)
            );
        }
        assert!(
            !out.windows(GPS_CANARY.len())
                .any(|w| w == GPS_CANARY.as_bytes()),
            "GPS canary leaked into PNG"
        );
        assert!(
            !out.windows(4).any(|w| w == b"Exif"),
            "Exif header leaked into PNG"
        );
        // And it still renders: decode back to the same dimensions.
        let frame = decode_to_frame(&out).unwrap();
        assert_eq!(frame.width(), 8);
        assert_eq!(frame.height(), 8);
    }

    #[test]
    fn png_text_chunks_stripped_on_reencode() {
        let dirty = png_with_text_chunks();
        let out = reencode_from_bytes(&dirty).expect("sanitize png");
        crate::png_chunks::assert_clean_png(&out).unwrap();
        let types = chunk_types(&out).unwrap();
        for ty in ["tEXt", "zTXt", "iTXt", "tIME", "eXIf"] {
            assert!(
                !types.iter().any(|t| t == ty.as_bytes()),
                "{ty} survived re-encode"
            );
        }
        for canary in [TEXT_CANARY, ZTXT_CANARY, ITXT_CANARY] {
            assert!(
                !out.windows(canary.len()).any(|w| w == canary.as_bytes()),
                "{canary} leaked"
            );
        }
        let chunks = parse_chunks(&out).unwrap();
        assert!(chunks.iter().any(|c| &c.ty == b"sRGB"), "sRGB tag required");
    }

    #[test]
    fn clean_png_roundtrip_is_stable_pixels() {
        let frame = solid_frame(1, 2, 3);
        let png = encode_clean_png(&frame).unwrap();
        let back = decode_to_frame(&png).unwrap();
        assert_eq!(frame, back);
    }
}
