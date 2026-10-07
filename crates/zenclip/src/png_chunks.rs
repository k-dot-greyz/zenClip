use crate::error::{Result, ZenclipError};

pub const PNG_SIGNATURE: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Chunks required or useful to *render* an 8-bit RGBA sRGB PNG.
/// Everything else is treated as metadata and must not appear in output.
pub const ALLOWED_CHUNKS: &[&[u8; 4]] = &[b"IHDR", b"IDAT", b"IEND", b"sRGB", b"gAMA", b"cHRM"];

/// Chunks that commonly leak identity, device, time, or location.
pub const FORBIDDEN_CHUNKS: &[&[u8; 4]] = &[
    b"tEXt", b"zTXt", b"iTXt", b"tIME", b"eXIf", b"iCCP", b"pHYs", b"bKGD", b"hIST", b"sPLT",
    b"sTER", b"dSIG",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PngChunk {
    pub ty: [u8; 4],
    pub data: Vec<u8>,
}

impl PngChunk {
    pub fn type_str(&self) -> &str {
        std::str::from_utf8(&self.ty).unwrap_or("????")
    }
}

/// Walk a PNG byte stream and return its chunks (CRC is trusted for tests;
/// we still require the signature and length bounds).
pub fn parse_chunks(png: &[u8]) -> Result<Vec<PngChunk>> {
    if png.len() < 8 || &png[..8] != PNG_SIGNATURE {
        return Err(ZenclipError::InvalidImage {
            format: "png",
            reason: "missing PNG signature".into(),
        });
    }
    let mut i = 8usize;
    let mut chunks = Vec::new();
    while i + 12 <= png.len() {
        let len = u32::from_be_bytes(png[i..i + 4].try_into().unwrap()) as usize;
        let ty: [u8; 4] = png[i + 4..i + 8].try_into().unwrap();
        let data_start = i + 8;
        let data_end = data_start
            .checked_add(len)
            .ok_or_else(|| ZenclipError::InvalidImage {
                format: "png",
                reason: "chunk length overflow".into(),
            })?;
        if data_end + 4 > png.len() {
            return Err(ZenclipError::InvalidImage {
                format: "png",
                reason: format!("truncated chunk {}", String::from_utf8_lossy(&ty)),
            });
        }
        chunks.push(PngChunk {
            ty,
            data: png[data_start..data_end].to_vec(),
        });
        i = data_end + 4; // skip CRC
        if &ty == b"IEND" {
            break;
        }
    }
    if chunks.is_empty() {
        return Err(ZenclipError::InvalidImage {
            format: "png",
            reason: "no chunks".into(),
        });
    }
    Ok(chunks)
}

pub fn chunk_types(png: &[u8]) -> Result<Vec<[u8; 4]>> {
    Ok(parse_chunks(png)?.into_iter().map(|c| c.ty).collect())
}

pub fn is_allowed_chunk(ty: &[u8; 4]) -> bool {
    ALLOWED_CHUNKS.iter().any(|a| a == &ty)
}

pub fn assert_clean_png(png: &[u8]) -> Result<()> {
    let chunks = parse_chunks(png)?;
    let unexpected: Vec<String> = chunks
        .iter()
        .filter(|c| !is_allowed_chunk(&c.ty))
        .map(|c| c.type_str().to_string())
        .collect();
    if !unexpected.is_empty() {
        return Err(ZenclipError::InvalidImage {
            format: "png",
            reason: format!("unexpected metadata chunks in output: {unexpected:?}"),
        });
    }
    let types: Vec<_> = chunks.iter().map(|c| c.ty).collect();
    if !types.contains(b"IHDR") || !types.contains(b"IDAT") || !types.contains(b"IEND") {
        return Err(ZenclipError::InvalidImage {
            format: "png",
            reason: "PNG missing IHDR/IDAT/IEND".into(),
        });
    }
    if !types.contains(b"sRGB") {
        return Err(ZenclipError::InvalidImage {
            format: "png",
            reason: "PNG missing sRGB chunk (colour not tagged)".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_png() {
        assert!(parse_chunks(b"not a png").is_err());
    }
}
