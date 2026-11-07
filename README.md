# scandisk

A Rust-based disk scanner to find and recover residues of deleted files from raw disk images and block devices.

## Prerequisites

To build and run this project you need to install Rust.

On Linux:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

On Windows go to the official [Rust website](https://rust-lang.org/tools/install/).

To verify the installation:

```bash
rustc --version
cargo --version
```

## Building the Project

You can compile the optimized release binary using the Makefile:

```bash
make build
```

*(Or compile a debug build with `make debug`).*

## How to Run (Using the Makefile)

A `Makefile` is provided to simplify running scans and extractions.

> **Note on Permissions:** When targeting physical raw block devices (e.g. `/dev/sda`), run commands with `sudo`:  
> `sudo make scan ...` or `sudo make carve ...`

### 1. Show Help

Print all available make targets and usage instructions:

```bash
make help
```

### 2. Scan-Only Mode

Scan a target without extracting files (counts and logs discovered files):

```bash
# Scan a disk image for all supported file types
# not specifying TYPE defaults to all
make scan TARGET=disk.img

# Scan a physical drive with sudo for specific types
sudo make scan TARGET=/dev/sda TYPE=png,jpeg
```

### 3. Carve (Extract) Mode

Scan and extract discovered files into a specified directory:

```bash
# Carve all files from an image
# not specifying TYPE defaults to all
make carve TARGET=disk.img OUT=./carved

# Carve specific file types from a USB drive
sudo make carve TARGET=/dev/sda OUT=./carved TYPE=png,jpeg
```

### 4. Shorthand Carve Commands

Convenience targets for quick single-format or complete extraction (one per supported file type plus `all`):

```bash
sudo make carve-all  TARGET=/dev/sda OUT=./carved
sudo make carve-png  TARGET=/dev/sda OUT=./carved
sudo make carve-jpeg TARGET=/dev/sda OUT=./carved
sudo make carve-pdf  TARGET=/dev/sda OUT=./carved
```

## Direct CLI Usage

You can also run the compiled binary directly without `make`:

```bash
./target/release/scandisk <TARGET> [OPTIONS]
```

Where `<TARGET>` (required) specifies the path to the raw disk image (`.img`, `.raw`) or block device (`/dev/sdX`).

Options are:

* `-o, --output-dir` (Optional): Directory where carved files will be saved. If omitted, runs in scan-only mode.
* `-t, --types` (Optional): Comma-separated list of file formats to search for. Supported types: `png`, `jpeg`, `pdf`, `all`. Defaults to `all`.

**Example:**

```bash
sudo ./target/release/scandisk /dev/sda -o ./carved_files -t png,pdf
```

Extract `png` and `pdf` files from the target `/dev/sda` and stores the retrieved files in `/carved_files`.

## Testing & Cleanup

Run unit tests:

```bash
make test
```

Remove build artifacts:

```bash
make clean
```
