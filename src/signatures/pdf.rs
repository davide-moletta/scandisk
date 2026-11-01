use super::FileCarver;

/// Default max size: 200 MB
const PDF_DEFAULT_MAX_SIZE: usize = 200 * 1024 * 1024;
pub const PDF_EXTENSION: &str = "pdf";
/// PDFs start with "%PDF-"
const PDF_HEADER: &[u8] = b"%PDF-";
const PDF_FOOTER: &[u8] = b"%%EOF";
const MIN_PDF_SIZE: usize = 32;

pub struct PdfCarver {
    max_size: usize,
}

impl PdfCarver {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Default for PdfCarver {
    fn default() -> Self {
        Self::new(PDF_DEFAULT_MAX_SIZE)
    }
}

impl FileCarver for PdfCarver {
    fn extension(&self) -> &str {
        PDF_EXTENSION
    }

    fn header(&self) -> &[u8] {
        PDF_HEADER
    }

    fn max_size(&self) -> usize {
        self.max_size
    }

    fn resolve_length(&self, data: &[u8]) -> Option<usize> {
        if data.len() < MIN_PDF_SIZE || !data.starts_with(PDF_HEADER) {
            return None;
        }

        let search_limit = data.len().min(self.max_size());
        let slice = &data[..search_limit];

        // If another PDF begins further down this slice, don't scan beyond it
        let effective_limit = match memchr::memmem::find(&slice[PDF_HEADER.len()..], PDF_HEADER) {
            Some(next_pdf_offset) => PDF_HEADER.len() + next_pdf_offset,
            None => slice.len(),
        };

        let active_slice = &slice[..effective_limit];

        // Find the LAST %%EOF marker within this document boundary
        let last_match = memchr::memmem::rfind(active_slice, PDF_FOOTER)?;
        let mut total_length = last_match + PDF_FOOTER.len();

        // Slurp any trailing newline or carriage return bytes after %%EOF
        while total_length < active_slice.len() {
            let b = active_slice[total_length];
            if b == b'\r' || b == b'\n' || b == b' ' {
                total_length += 1;
            } else {
                break;
            }
        }

        Some(total_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pdf_carve() {
        let carver = PdfCarver::default();

        // Valid PDF payload ending with %%EOF\r\n
        let valid_pdf =
            b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\nxref\n0 1\ntrailer\n<<>>\nstartxref\n9\n%%EOF\r\n";
        let trailing_junk = b"\x00\x00\xFF\xAA";

        // Concatenate valid PDF with disk garbage
        let mut disk_data = valid_pdf.to_vec();
        disk_data.extend_from_slice(trailing_junk);

        let result = carver.resolve_length(&disk_data);

        // It should carve exactly the valid PDF length (70 bytes), ignoring trailing garbage
        assert_eq!(result, Some(valid_pdf.len()));
    }

    #[test]
    fn test_too_small_pdf_data() {
        let carver = PdfCarver::default();
        let short_data = b"%PDF-1"; // Less than MIN_PDF_SIZE (32)
        assert_eq!(carver.resolve_length(short_data), None);
    }
}
