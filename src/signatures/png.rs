use super::FileCarver;

/// Default max size: 50 MB
const PNG_DEFAULT_MAX_SIZE: usize = 50 * 1024 * 1024;
const PNG_EXTENSION: &str = "png";
/// PNG header and footer marks
const PNG_HEADER: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
const PNG_FOOTER: &[u8] = &[0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];
/// Smallest possible valid PNG is 67 bytes (Header + IHDR + IDAT + IEND)
const PNG_MIN_SIZE: usize = 67;

pub struct PngCarver {
    max_size: usize,
}

impl PngCarver {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

/// Default builder for [`PngCarver`]
impl Default for PngCarver {
    fn default() -> Self {
        Self::new(PNG_DEFAULT_MAX_SIZE)
    }
}

/// Implementation of [`FileCarver`] for PNG files
impl FileCarver for PngCarver {
    fn extension(&self) -> &str {
        PNG_EXTENSION
    }

    fn header(&self) -> &[u8] {
        PNG_HEADER
    }

    fn max_size(&self) -> usize {
        self.max_size
    }

    fn resolve_length(&self, data: &[u8]) -> Option<usize> {
        // Data must be at least the minimum size and start with the PNG signature
        if data.len() < PNG_MIN_SIZE || !data.starts_with(PNG_HEADER) {
            return None;
        }

        // Bound the search window to max_size
        let search_limit = data.len().min(self.max_size());
        let slice = &data[..search_limit];

        // Search for the footer marker
        // Safe to slice because we already asserted data.len() >= PNG_MIN_SIZE
        let search_start = PNG_MIN_SIZE - PNG_FOOTER.len();
        let footer_offset = memchr::memmem::find(&slice[search_start..], PNG_FOOTER)?;

        let total_length = search_start + footer_offset + PNG_FOOTER.len();
        Some(total_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_png_carve() {
        let carver = PngCarver::default();

        // 67-byte minimal valid 1x1 PNG + trailing garbage
        let sample_png = [
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // Signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
            0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, // IDAT
            0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D,
            0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, // IEND
            0x42, 0x60, 0x82, 0xFF, 0xFF, 0xFF, // Trailing junk data from disk
        ];

        let result = carver.resolve_length(&sample_png);
        assert_eq!(result, Some(67));
    }

    #[test]
    fn test_too_small_png_data() {
        let carver = PngCarver::default();
        let short_data = [0x89, 0x50, 0x4E, 0x47]; // Less than PNG_MIN_SIZE
        assert_eq!(carver.resolve_length(&short_data), None);
    }
}
