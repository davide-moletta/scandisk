# scandisk

A Rust-based disk scanner to find and recover residues of deleted files from raw disk images and block devices.

## Installation & Prerequisites

You can run `scandisk` either natively via Rust or inside a Docker container (no local Rust toolchain required).

### Option 1: Native Rust Installation

Install the Rust toolchain:

* **Linux / macOS:**

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

* **Windows:** Download the installer from the official [Rust website](https://rust-lang.org/tools/install/).

Verify installation:

```bash
rustc --version
cargo --version
```

### Option 2: Docker Installation

If you don't have Rust installed, you can build the Docker container using `make`:

```bash
make docker-build
```

## How to Run (Using the Makefile)

A `Makefile` is provided to manage both native execution and Docker runs with zero configuration.

> **Note on Permissions:** When targeting raw physical block devices (e.g. `/dev/sda`), run the commands with `sudo`:  
> `sudo make scan ...` or `sudo make docker-carve ...`

### 1. Show Help Menu

Print all available make targets and usage instructions:

```bash
make help
```

### 2. Native Execution (Requires Rust)

Compile the optimized binary first:

```bash
make build
```

#### Scan-Only Mode

Scan a target without extracting files (detects and logs file boundaries and totals):

```bash
# Scan a disk image file for all supported formats
make scan TARGET=disk.img

# Scan a USB drive for specific file types
sudo make scan TARGET=/dev/sda TYPE=png,jpeg
```

#### Carve (Extract) Mode

Scan and extract discovered files into a destination directory:

```bash
# Carve all files from an image
make carve TARGET=disk.img OUT=./carved_files

# Carve specific file types from a physical drive
sudo make carve TARGET=/dev/sda OUT=./carved_files TYPE=png,jpeg
```

#### Shorthand Carve Commands

Quick commands for specific file formats:

```bash
sudo make carve-all  TARGET=/dev/sda OUT=./carved_all
sudo make carve-png  TARGET=/dev/sda OUT=./carved_pngs
sudo make carve-jpeg TARGET=/dev/sda OUT=./carved_jpegs
sudo make carve-pdf  TARGET=/dev/sda OUT=./carved_pdfs
```

### 3. Docker Execution (No Local Rust Required)

Docker targets automatically mount your current directory and hardware block devices, allowing you to use local paths directly.

```bash
# Build the Docker image
make docker-build

# Scan a disk image
make docker-scan TARGET=disk.img

# Carve specific file types from a disk image
make docker-carve TARGET=disk.img OUT=./carved_files TYPE=png,jpeg

# Carve from a physical USB stick (requires sudo)
sudo make docker-carve TARGET=/dev/sda OUT=./carved_files
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
