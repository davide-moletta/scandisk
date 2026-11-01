pub mod jpeg;
pub mod pdf;
pub mod png;

/// Generic trait implemented for each file type
pub trait FileCarver: Send + Sync {
    /// Human-readable format name and extension
    fn extension(&self) -> &str;

    /// The magic header bytes to scan for
    fn header(&self) -> &[u8];

    /// Sanity limit to prevent runaway memory reads if footer is missing
    fn max_size(&self) -> usize;

    /// Inspects the byte slice starting right at the header
    /// and returns the total byte length of the file if valid.
    fn resolve_length(&self, data: &[u8]) -> Option<usize>;
}
