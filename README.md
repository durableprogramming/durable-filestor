# durable-filestor

A robust file storage system for reading and writing to various file storage backends. Built with Rust for reliability, performance, and long-term maintainability.

## Architecture

durable-filestor is organized into three main components:

- **`apps/core`** - Core CLI and configuration system that provides a unified interface and translates custom protocols to service-specific commands
- **`apps/services`** - Individual storage service implementations, each with their own CLI
- **`apps/shared`** - Common code, macros, and utilities shared across all components

## Features

### Core Functionality
- Unified interface for multiple file storage systems
- Custom protocol support (e.g., `my-s3://my-bucket/bob.txt`)
- Flexible configuration through multiple sources
- Service discovery and command translation

### Storage Operations
All storage services support these standard operations:
- **list** - List files and directories
- **upload** - Upload files to storage
- **download** - Download files from storage
- **head** - Retrieve file metadata without downloading

Additional operations available in some services:
- **delete** - Delete files (S3, Memory)
- **copy** - Copy files within storage (S3, Memory)
- **stats** - View storage statistics (Memory)
- **cleanup** - Remove expired entries (Memory)

## Configuration

durable-filestor supports multiple configuration methods, processed in order of precedence:

1. Command line switches (highest priority)
2. Environment variables (with `DURABLE_FILESTOR_` prefix)
3. Configuration file (lowest priority)
   - Custom file via `--config` flag, OR
   - Default user config file: `~/.config/durable-filestor/filestor.toml`

Note: The `--config` flag determines which file to load, but the file's contents always have the lowest priority compared to environment variables and CLI arguments.

### Custom Protocols

Configure custom protocols in your TOML file to simplify common operations:

```toml
[protocols]
my-s3 = { service = "s3", bucket = "my-default-bucket", region = "us-east-1" }
backup = { service = "local", path = "/backup/storage" }
```

### Key Translation Options

Configure per-protocol key transformation options to modify how file keys are processed:

```toml
[protocols]
# Protocol with case-insensitive keys
docs = { 
    service = "s3", 
    bucket = "company-documents", 
    region = "us-west-2",
    key_translation = { downcase = true }
}

# Protocol with hashed and partitioned keys
media = { 
    service = "s3", 
    bucket = "media-storage",
    key_translation = { 
        hash_function = "sha256",
        partition = true,
        partition_length = 4,
        partition_depth = 2
    }
}

# Protocol with prefixed and normalized keys
backup = { 
    service = "local", 
    path = "/backup/storage",
    key_translation = {
        prefix = "backup/",
        suffix = ".bak",
        normalize_unicode = true,
        replace_spaces = "_"
    }
}
```

#### Available Key Translation Options

```toml
key_translation = {
    # Case transformation (mutually exclusive)
    downcase = true,              # Convert keys to lowercase
    upcase = false,               # Convert keys to uppercase
    
    # Hashing options
    hash_function = "sha256",     # Hash keys: md5, sha1, sha256, sha512, blake2b
    hash_prefix_length = 8,       # Use only first N characters of hash
    
    # Key partitioning for directory structure
    partition = true,             # Enable key partitioning
    partition_length = 4,         # Characters per partition segment
    partition_depth = 2,          # Number of partition levels
    partition_separator = "/",    # Directory separator
    
    # Prefix and suffix options
    prefix = "files/",            # Add prefix to all keys
    suffix = ".dat",              # Add suffix to all keys
    timestamp_suffix = false,     # Add timestamp suffix
    
    # Normalization options
    normalize_unicode = true,     # Normalize unicode characters
    strip_whitespace = true,      # Remove leading/trailing whitespace
    replace_spaces = "_",         # Replace spaces with character
    max_key_length = 255          # Truncate long keys
}
```

Example transformations:
- `docs://My File.txt` → stored as `my file.txt`
- `media://document.pdf` → stored as `a1b2/c3d4/document.pdf` 
- `backup://report.doc` → stored as `backup/report.doc.bak`

Then use them directly:
```bash
# Instead of: durable-filestor-s3 --bucket my-default-bucket --region us-east-1 upload file.txt
durable-filestor upload my-s3://file.txt local-file.txt

# Instead of: durable-filestor-local --path /backup/storage list
durable-filestor list backup://
```

## Usage

### Core CLI (Work in Progress)
```bash
# Configure a new storage protocol (NOT YET IMPLEMENTED)
durable-filestor config add-protocol my-s3 --service s3 --bucket mybucket

# Use configured protocols (NOT YET IMPLEMENTED)
durable-filestor list my-s3://documents/
durable-filestor upload my-s3://documents/report.pdf ./report.pdf
durable-filestor download my-s3://documents/report.pdf ./downloaded-report.pdf
```

### Direct Service Usage
```bash
# Use services directly with their own binaries
durable-filestor-s3 --bucket mybucket list documents/
durable-filestor-local --path /storage upload ./file.txt remote-file.txt
durable-filestor-memory --port 8080 stats
```

### Development Utilities
```bash
# Test and debug configuration parsing
dfs-config-toml-parse --file config.toml
dfs-config-toml-export --output exported.toml

# Test and debug URI parsing
dfs-uri-test-parse my-s3://bucket/key.txt
dfs-config-uri-test-parse --config-uri "s3://bucket?region=us-east-1"
dfs-config-uri-export --protocol my-s3
```

## Installation

### From Source
```bash
git clone https://github.com/your-org/durable-filestor.git
cd durable-filestor
cargo build --release
```

### Using Cargo
```bash
cargo install durable-filestor-core
```

## Supported Storage Systems

- **Local filesystem** - Direct file system access
- **Amazon S3** - S3-compatible object storage with delete and copy operations
- **Memory** - In-memory storage with TTL support, stats, and cleanup operations
- **Additional services** - Extensible architecture for adding new storage backends

## Configuration File Example

```toml
# ~/.config/durable-filestor/filestor.toml

[protocols]
docs = { service = "s3", bucket = "company-documents", region = "us-west-2" }
backup = { service = "local", path = "/mnt/backup" }
temp = { service = "s3", bucket = "temp-storage", region = "us-east-1" }

[services.s3]
default_region = "us-east-1"
endpoint = "https://s3.amazonaws.com"

[services.local]
default_permissions = "644"
```

## Environment Variables

All configuration options can be set via environment variables using the `DURABLE_FILESTOR_` prefix:

```bash
export DURABLE_FILESTOR_S3_REGION=us-west-2
export DURABLE_FILESTOR_S3_BUCKET=my-default-bucket
```

## Development

### Project Structure
```
apps/
├── core/                      # Core CLI and protocol management
│   ├── uri_parser/            # Custom protocol URI parsing
│   ├── config_toml_parser/    # TOML configuration parsing
│   ├── config_toml_exporter/  # TOML configuration export
│   ├── config_uri_parser/     # URI-based configuration parsing
│   └── config_uri_exporter/   # URI-based configuration export
├── services/                  # Storage service implementations
│   ├── s3/                    # Amazon S3 storage
│   ├── local/                 # Local filesystem storage
│   └── memory/                # In-memory storage with TTL
└── shared/                    # Common utilities and macros
```

### Building
```bash
# Build all components
cargo build

# Build specific component
cargo build -p durable-filestor-core
cargo build -p durable-filestor-s3
cargo build -p durable-filestor-local
cargo build -p durable-filestor-memory
cargo build -p durable-filestor-uri-parser
cargo build -p durable-filestor-config-toml-parser
cargo build -p durable-filestor-config-toml-exporter
cargo build -p durable-filestor-config-uri-parser
cargo build -p durable-filestor-config-uri-exporter
```

### Testing
```bash
# Run all tests
cargo test

# Test specific service
cargo test -p durable-filestor-core
cargo test -p durable-filestor-s3
cargo test -p durable-filestor-local
cargo test -p durable-filestor-memory
cargo test -p durable-filestor-uri-parser
cargo test -p durable-filestor-config-toml-parser
cargo test -p durable-filestor-config-toml-exporter
cargo test -p durable-filestor-config-uri-parser
cargo test -p durable-filestor-config-uri-exporter
```

## Philosophy

durable-filestor is built with our core principles in mind:

- **Reliability** - Robust error handling and data integrity
- **Simplicity** - Clear, consistent interface across all storage types
- **Maintainability** - Modular architecture for long-term sustainability
- **Pragmatism** - Practical solutions for real-world file storage needs

## License

[License information]

## Contributing

We welcome contributions that align with our philosophy of building durable, practical software. Please see [CONTRIBUTING.md] for guidelines.
