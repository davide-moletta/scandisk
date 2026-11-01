use super::FileCarver;

/// Default max size: 50 MB
const JPEG_DEFAULT_MAX_SIZE: usize = 50 * 1024 * 1024;
pub const JPEG_EXTENSION: &str = "jpg";
/// JPEG header and footer marks
const JPEG_HEADER: &[u8] = &[0xFF, 0xD8, 0xFF];
const JPEG_FOOTER: &[u8] = &[0xFF, 0xD9];
/// Smallest possible valid JPEG is 128 bytes
const MIN_JPEG_SIZE: usize = 128;

pub struct JpegCarver {
    max_size: usize,
}

impl JpegCarver {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Default for JpegCarver {
    fn default() -> Self {
        Self::new(JPEG_DEFAULT_MAX_SIZE)
    }
}

impl FileCarver for JpegCarver {
    fn extension(&self) -> &str {
        JPEG_EXTENSION
    }

    fn header(&self) -> &[u8] {
        JPEG_HEADER
    }

    fn max_size(&self) -> usize {
        self.max_size
    }

    fn resolve_length(&self, data: &[u8]) -> Option<usize> {
        if data.len() < MIN_JPEG_SIZE || !data.starts_with(JPEG_HEADER) {
            return None;
        }

        let search_limit = data.len().min(self.max_size());
        let slice = &data[..search_limit];

        // Walk past metadata segments (APPn, DQT, DHT, etc.) to skip EXIF thumbnails
        // Every marker between SOI (0xFFD8) and SOS (0xFFDA) has a 2-byte big-endian length
        let mut offset = 2; // Start immediately after FF D8
        let mut scan_start = MIN_JPEG_SIZE;

        while offset + 4 <= slice.len() {
            if slice[offset] != 0xFF {
                break;
            }

            let marker = slice[offset + 1];

            // SOS (Start of Scan 0xDA): Image data stream begins here
            if marker == 0xDA {
                // Read SOS header length
                let sos_len = u16::from_be_bytes([slice[offset + 2], slice[offset + 3]]) as usize;
                scan_start = offset + 2 + sos_len;
                break;
            }

            // Standalone markers without a payload length:
            // RST0-RST7 (0xD0-0xD7) or SOI (0xD8)
            if (0xD0..=0xD8).contains(&marker) {
                offset += 2;
                continue;
            }

            // Variable length marker segment: read big-endian length (includes length bytes)
            let seg_len = u16::from_be_bytes([slice[offset + 2], slice[offset + 3]]) as usize;
            if seg_len < 2 || offset + 2 + seg_len > slice.len() {
                break;
            }
            offset += 2 + seg_len;
        }

        // Search for the terminal FF D9 footer after the SOS marker
        let search_start = scan_start.min(slice.len().saturating_sub(JPEG_FOOTER.len()));
        let footer_offset = memchr::memmem::find(&slice[search_start..], JPEG_FOOTER)?;

        let total_length = search_start + footer_offset + JPEG_FOOTER.len();
        Some(total_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_jpeg_carve() {
        let carver = JpegCarver::default();

        // Synthetic minimal JPEG structure
        let mut sample = vec![
            0xFF, 0xD8, 0xFF, 0xE0, // SOI, APP0
            0x00, 0x10, // APP0 length (16 bytes)
            0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00,
            0xFF, 0xDA, // SOS (Start of Scan)
            0x00, 0x08, // SOS length (8 bytes)
            0x01, 0x01, 0x00, 0x00, 0x3F, 0x00,
        ];
        // Add image payload up to minimum size
        sample.resize(130, 0xAB);
        // Append EOI (End of Image)
        sample.extend_from_slice(&[0xFF, 0xD9]);
        // Add trailing garbage
        sample.extend_from_slice(&[0x00, 0x00, 0x00]);

        let result = carver.resolve_length(&sample);
        assert_eq!(result, Some(132));
    }
}
