# Crepe 🎨

A grep-like text search tool that **highlights matches instead of filtering lines**. Built in Rust for speed and reliability.

Unlike traditional grep which hides non-matching lines, crepe shows you the full context with matches beautifully highlighted in color. Perfect for reading logs, searching code, and understanding data in context.

## Features

- 🌈 **Multiple pattern highlighting** - Each pattern gets a different color
- 🔍 **Full regex support** - All the power of Rust's regex engine
- ⚡ **Parallel processing** - Automatically uses all CPU cores for multi-file searches
- 📁 **Smart file discovery** - Respects .gitignore, filters by patterns, skips binaries
- 🎯 **Context-aware** - Show lines before/after matches (like grep's `-C`)
- 📊 **Multiple output modes** - Normal, count-only, only-matching, inverted
- 🔧 **Highly configurable** - But simple by default

## Installation

### Download Pre-built Binary

Download the latest release for your platform:

**Linux (amd64)**
```bash
curl -L https://github.com/brunoribeiro/crepe/releases/latest/download/crepe-VERSION-linux-amd64 -o crepe
chmod +x crepe
sudo mv crepe /usr/local/bin/
```

**macOS (Apple Silicon/ARM)**
```bash
curl -L https://github.com/brunoribeiro/crepe/releases/latest/download/crepe-VERSION-macos-arm64 -o crepe
chmod +x crepe
sudo mv crepe /usr/local/bin/
```

**macOS (Intel)**
```bash
curl -L https://github.com/brunoribeiro/crepe/releases/latest/download/crepe-VERSION-macos-intel -o crepe
chmod +x crepe
sudo mv crepe /usr/local/bin/
```

Replace `VERSION` with the latest version number (e.g., `0.1.0`) from the [releases page](https://github.com/brunoribeiro/crepe/releases).

### From source (requires Rust)

```bash
git clone <repository-url>
cd crepe
cargo install --path .
```

The `crepe` binary will be installed to `~/.cargo/bin/`.

### Quick test without installing

```bash
cargo run -- [OPTIONS] PATTERN [FILE...]
```

## Usage

### Basic Examples

```bash
# Highlight all occurrences of "error" in a file
crepe "error" app.log

# Case-insensitive search
crepe -i "warning" app.log

# Multiple patterns with different colors
crepe -e TODO -e FIXME -e HACK src/

# Search recursively through a directory
crepe -r "function" src/
```

### Advanced Examples

```bash
# Show line numbers and 2 lines of context
crepe -n -C 2 "fn main" src/

# Count matches in all Rust files
crepe -r -c --include "*.rs" "struct" .

# Search hidden files, exclude node_modules
crepe -r --hidden --exclude "node_modules/*" "import" .

# Only show the matched parts (not full lines)
crepe -o "\d+\.\d+\.\d+" CHANGELOG.md

# Show lines that DON'T match the pattern
crepe -v "debug" app.log

# Match whole words only
crepe -w "cat" file.txt  # matches "cat" but not "concatenate"

# Limit to first 10 matches
crepe -m 10 "error" large-file.log
```

### Regex Examples

```bash
# Email addresses
crepe "\w+@\w+\.\w+" contacts.txt

# IP addresses
crepe "\d+\.\d+\.\d+\.\d+" server.log

# Function definitions in JavaScript
crepe "function \w+\(" src/*.js

# Hexadecimal colors
crepe "#[0-9a-fA-F]{6}" styles.css
```

## Command-Line Options

### Pattern Matching
- `-e, --regexp PATTERN` - Specify multiple patterns (can be used multiple times)
- `-i, --ignore-case` - Case-insensitive matching
- `-w, --word-regexp` - Match whole words only
- `-v, --invert-match` - Show lines that DON'T match

### Display Options
- `-n, --line-number` - Show line numbers
- `-o, --only-matching` - Show only the matched parts
- `-c, --count` - Show only count of matching lines
- `-A NUM` - Show NUM lines after each match
- `-B NUM` - Show NUM lines before each match
- `-C NUM` - Show NUM lines before and after each match

### File Discovery
- `-r, --recursive` - Search directories recursively
- `--include GLOB` - Only search files matching pattern (e.g., `*.rs`)
- `--exclude GLOB` - Exclude files matching pattern
- `--hidden` - Search hidden files and directories
- `--max-depth NUM` - Maximum directory depth
- `--follow` - Follow symbolic links
- `--binary` - Search binary files (skipped by default)

### Output Control
- `-H, --with-filename` - Show filename with output
- `--no-filename` - Don't show filename
- `--heading` - Group output by filename

### Performance
- `-m, --max-count NUM` - Stop after NUM matches
- `-j, --jobs NUM` - Number of threads (default: auto)

### Colors
- `--color WHEN` - When to use colors (auto/always/never)
- `--no-color` - Disable colors

## How It Works

Crepe is built with a modular architecture:

- **Pattern matching**: Uses Rust's `regex` crate for fast, reliable regex
- **Highlighting**: Terminal colors via `colored` crate - 12 colors that cycle for multiple patterns
- **File discovery**: `ignore` crate for gitignore support, `walkdir` for traversal
- **Parallelization**: `rayon` for automatic multi-core processing
- **CLI parsing**: `clap` for robust argument handling

### Color Palette

Each pattern gets a unique color in this order:
1. Red
2. Green
3. Yellow
4. Blue
5. Magenta
6. Cyan
7. Bright Red
8. Bright Green
9. Bright Yellow
10. Bright Blue
11. Bright Magenta
12. Bright Cyan

After 12 patterns, colors cycle back to the beginning.

## Comparison with grep

| Feature | grep | crepe |
|---------|------|-------|
| Filter non-matching lines | ✅ (default) | ❌ Shows all lines |
| Highlight matches | ✅ (--color) | ✅ (default) |
| Multiple pattern colors | ❌ One color | ✅ Different color per pattern |
| Parallel processing | ❌ | ✅ Automatic |
| Respects .gitignore | ❌ | ✅ Automatic |
| Binary file detection | ✅ | ✅ |
| Context lines | ✅ | ✅ |
| Regex support | ✅ | ✅ |

**Use grep when**: You want to filter and reduce output
**Use crepe when**: You want to see everything with highlights

## Development

```bash
# Run tests
cargo test

# Run with debug output
cargo run -- pattern file

# Build release version
cargo build --release

# Run linter
cargo clippy

# Format code
cargo fmt
```

## Examples in Action

### Log Analysis
```bash
# Highlight different log levels in different colors
crepe -e ERROR -e WARN -e INFO app.log
```

### Code Search
```bash
# Find all TODO/FIXME/HACK comments in your codebase
crepe -r -n -e TODO -e FIXME -e HACK src/
```

### Configuration Review
```bash
# Find all uncommented lines in a config file
crepe -v "^#" config.conf
```

### Data Extraction
```bash
# Extract all URLs from a file
crepe -o "https?://[^\s]+" document.txt
```

## Built By

**Built by Claude** (Anthropic) - An AI assistant that writes production-quality code.

This entire tool, from architecture to implementation to documentation, was created through an interactive conversation with Claude. The result is a well-structured, tested, and fully-functional Rust application built in minutes.

## License

This project is provided as-is for educational and practical use.

## Contributing

Feel free to open issues or submit pull requests!

## Acknowledgments

- Built with ❤️ and Rust 🦀
- Powered by industry-standard crates: clap, regex, rayon, ignore, colored
- Inspired by grep, ripgrep, and the need for better context in search results
