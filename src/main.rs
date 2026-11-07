mod signatures;

use signatures::{FileCarver, jpeg, pdf, png};

use aho_corasick::{AhoCorasick, MatchKind};
use clap::{Parser, ValueEnum};
use log::{error, info, warn};
use memmap2::MmapOptions;
use std::{
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FileType {
    Jpeg,
    Pdf,
    Png,
    All,
}

impl FileType {
    /// Inspects the requested CLI types, handles `All`, and constructs
    /// a deduplicated list of active carver instances.
    pub fn build_carvers(types: &[FileType]) -> Vec<Box<dyn FileCarver>> {
        // If "All" was passed, return one of each supported carver
        if types.contains(&FileType::All) {
            return vec![
                Box::new(png::PngCarver::default()),
                Box::new(jpeg::JpegCarver::default()),
                Box::new(pdf::PdfCarver::default()),
            ];
        }

        let mut carvers: Vec<Box<dyn FileCarver>> = Vec::new();

        for t in types {
            match t {
                FileType::Png => {
                    // Avoid duplicate instances if user passed `-t png -t png`
                    if !carvers.iter().any(|c| c.extension() == png::PNG_EXTENSION) {
                        carvers.push(Box::new(png::PngCarver::default()));
                    }
                }
                FileType::Jpeg => {
                    if !carvers
                        .iter()
                        .any(|c| c.extension() == jpeg::JPEG_EXTENSION)
                    {
                        carvers.push(Box::new(jpeg::JpegCarver::default()));
                    }
                }
                FileType::Pdf => {
                    if !carvers.iter().any(|c| c.extension() == pdf::PDF_EXTENSION) {
                        carvers.push(Box::new(pdf::PdfCarver::default()));
                    }
                }
                FileType::All => unreachable!(),
            }
        }

        carvers
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "scandisk",
    about = "Tool to scan raw disk images or block devices to reconstruct files via magic bytes."
)]
pub struct Args {
    /// Path to the raw disk image (.img, .raw) or block device (/dev/sdX)
    #[arg(required = true)]
    pub image_path: PathBuf,

    /// Output directory for carved files (if omitted, runs in scan-only mode)
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,

    /// Target file types to scan for
    #[arg(short, long, value_enum, value_delimiter = ',', default_value = "all")]
    pub types: Vec<FileType>,
}

/// Writes carved file bytes directly to the output directory
fn extract_file(
    output_dir: &Path,
    extension: &str,
    offset: usize,
    data: &[u8],
) -> std::io::Result<PathBuf> {
    let filename = format!("carved_0x{:08X}.{}", offset, extension);
    let destination = output_dir.join(filename);

    let mut file = File::create(&destination)?;
    file.write_all(data)?;
    Ok(destination)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let args = Args::parse();

    info!("Target: {:?}", args.image_path);

    // Verify target exists and determine actual byte capacity
    let mut file = File::open(&args.image_path)?;
    let mut total_bytes = file.metadata()?.len();

    // On Linux, block devices report 0 from metadata(), so seek to the end to get capacity
    if total_bytes == 0 {
        total_bytes = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?; // Rewind back to the start
    }

    if total_bytes == 0 {
        warn!("Target is 0 bytes, nothing to scan");
        return Ok(());
    }

    info!(
        "Target size: {:.2} MB",
        total_bytes as f64 / (1024.0 * 1024.0)
    );

    // Memory-map using the explicit target length
    let mmap = unsafe { MmapOptions::new().len(total_bytes as usize).map(&file)? };

    // Prepare output directory if carving is enabled
    if let Some(ref out_dir) = args.output_dir {
        fs::create_dir_all(out_dir)?;
        info!("Carving enabled, files will be saved to: {:?}", out_dir);
    } else {
        info!("Running in scan-only mode");
    }

    // Instantiate active carvers
    let carvers = FileType::build_carvers(&args.types);
    if carvers.is_empty() {
        warn!("No active carvers selected");
        return Ok(());
    }

    info!(
        "Active carvers: {:?}",
        carvers.iter().map(|c| c.extension()).collect::<Vec<_>>()
    );

    // Build Aho-Corasick multi-pattern search automaton
    // Collect all headers in the exact same order as `carvers`
    let patterns: Vec<&[u8]> = carvers.iter().map(|c| c.header()).collect();

    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostFirst)
        .build(&patterns)?;

    info!(
        "Compiled search automaton for {} file types, beginning scan...",
        patterns.len()
    );

    // Single-pass linear scan across the whole drive
    let mut files_found = 0;
    let mut cursor = 0;

    while cursor < mmap.len() {
        // Find the next occurrence of any active header from cursor position
        if let Some(mat) = ac.find(&mmap[cursor..]) {
            let header_offset = cursor + mat.start();
            let carver_idx = mat.pattern().as_usize();
            let carver = &carvers[carver_idx];

            let candidate_slice = &mmap[header_offset..];

            // Attempt to resolve file boundary
            if let Some(length) = carver.resolve_length(candidate_slice) {
                let end_offset = header_offset + length;

                // Bounds safety check (e.g. truncated file at the end of the disk)
                if end_offset > mmap.len() {
                    warn!(
                        "File at 0x{:08X} exceeds buffer bounds (len: {}), skipping",
                        header_offset, length
                    );
                    cursor = header_offset + 1;
                    continue;
                }

                files_found += 1;
                let file_bytes = &mmap[header_offset..end_offset];

                info!(
                    "Found valid {} at 0x{:08X} - 0x{:08X} ({} bytes)",
                    carver.extension(),
                    header_offset,
                    end_offset,
                    length
                );

                // Extract file if output directory is provided
                if let Some(ref out_dir) = args.output_dir {
                    match extract_file(out_dir, carver.extension(), header_offset, file_bytes) {
                        Ok(saved_path) => info!("Carved: {:?}", saved_path),
                        Err(e) => error!("Failed to write 0x{:08X}: {}", header_offset, e),
                    }
                }

                // Skip scanning inside this file's payload
                cursor = end_offset;
            } else {
                // Header matched, but carver determined it's a false positive.
                // Advance by 1 byte to catch any adjacent or overlapping headers.
                cursor = header_offset + 1;
            }
        } else {
            // No more magic headers exist on the rest of the disk
            break;
        }
    }

    info!("Scan complete, total files recovered: {}", files_found);

    Ok(())
}
