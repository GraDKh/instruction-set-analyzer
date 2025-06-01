# Instruction Set Validator

This utility analyzes x86_64 ELF binaries to determine which CPU instruction sets (such as SSE, AVX, AVX2, AVX-512, etc.) are used by the binary. It is useful for verifying binary compatibility with target CPUs, especially when deploying to environments with varying hardware capabilities.

## Features
- Detects and lists all CPU instruction sets used by a given binary.
- Accepts either a single binary file or a directory.
- If a directory is specified, recursively finds all executable files and reports the union of all instruction sets used.
- Supports Linux (uses Unix file permissions to detect executables).

## Getting Started

### Downloading the Sources

Clone the repository:

```sh
git clone <repository-url>
cd instruction_set_validator
```

Replace `<repository-url>` with the actual URL of the repository if not already cloned.

### Building

Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed (edition 2021 or newer).

Build the project in release mode:

```sh
cargo build --release
```

The compiled binary will be located in `target/release/instruction-set-validator`.

## Usage

```sh
cargo run --release -- <path-to-binary-or-folder>
```
- If you provide a file, it will print the instruction sets used by that binary.
- If you provide a directory, it will recursively scan for executables and print the union of all instruction sets used.

## Example Output
```
Binary ./my_binary uses the following CPU features:
  SSE2
  AVX
  AVX2
```
Or for a directory:
```
All binaries in ./bin/ use the following CPU features (union):
  SSE2
  AVX
  AVX2
  AVX512F
```

## Requirements
- Rust (edition 2021)
- Linux (for directory scanning and executable detection)

## License
MIT
