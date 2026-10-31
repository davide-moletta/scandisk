use clap::{Parser, ValueEnum};
use log::info;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum FileType {
    Jpg,
    Pdf,
    Png,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReportFormat {
    Text,
    Json,
    Csv,
}

#[derive(Parser, Debug)]
#[command(
    name = "scandisk",
    long_about = "Tool to scan raw disk images or block devices to reconstruct files via magic bytes."
)]
pub struct Args {
    // ==================== Target ====================
    /// Path to the raw disk image (.img, .raw) or block device (/dev/sdX)
    #[arg(required = true)]
    pub image_path: PathBuf,

    // ==================== Output & Extraction ====================
    /// Output directory for carved files (if omitted, runs in scan-only mode)
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,

    /// Target file types to scan for
    #[arg(short, long, value_enum, default_value = "all")]
    pub types: Vec<FileType>,

    /// Path to output a report
    #[arg(short, long)]
    pub report: Option<PathBuf>,

    /// Format for the report
    #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
    pub format: ReportFormat,
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let args = Args::parse();

    info!("Target Image: {:?}", args.image_path);
    info!("Target Types: {:?}", args.types);

    if let Some(ref out) = args.output_dir {
        info!("Carving enabled, writing files to: {:?}", out);
    } else {
        info!("Running in scan-only mode (pass -o <DIR> to carve files)");
    }
}
