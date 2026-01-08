use clap::Parser;
use std::path::PathBuf;

use crate::config::{ColorMode, Config};

/// Crepe - A grep-like tool that highlights matches instead of filtering lines
#[derive(Parser, Debug)]
#[command(name = "crepe")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Pattern to search for (regex)
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Files to search (or stdin if not specified)
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    /// Additional patterns to search for
    #[arg(short = 'e', long = "regexp", value_name = "PATTERN")]
    pub patterns: Vec<String>,

    /// Case-insensitive matching
    #[arg(short = 'i', long = "ignore-case")]
    pub ignore_case: bool,

    /// Only match whole words
    #[arg(short = 'w', long = "word-regexp")]
    pub word_regexp: bool,

    /// Invert match (show lines that don't match)
    #[arg(short = 'v', long = "invert-match")]
    pub invert_match: bool,

    /// Show line numbers
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Show only the matched parts
    #[arg(short = 'o', long = "only-matching")]
    pub only_matching: bool,

    /// Show only count of matching lines
    #[arg(short = 'c', long = "count")]
    pub count: bool,

    /// Lines of context before matches
    #[arg(short = 'B', long = "before-context", value_name = "NUM", default_value = "0")]
    pub before_context: usize,

    /// Lines of context after matches
    #[arg(short = 'A', long = "after-context", value_name = "NUM", default_value = "0")]
    pub after_context: usize,

    /// Lines of context before and after matches
    #[arg(short = 'C', long = "context", value_name = "NUM")]
    pub context: Option<usize>,

    /// Search directories recursively
    #[arg(short = 'r', long = "recursive")]
    pub recursive: bool,

    /// Follow symbolic links
    #[arg(long = "follow")]
    pub follow: bool,

    /// Search hidden files and directories
    #[arg(long = "hidden")]
    pub hidden: bool,

    /// Maximum depth for recursive search
    #[arg(long = "max-depth", value_name = "NUM")]
    pub max_depth: Option<usize>,

    /// Only search files matching glob pattern
    #[arg(long = "include", value_name = "GLOB")]
    pub include: Vec<String>,

    /// Exclude files matching glob pattern
    #[arg(long = "exclude", value_name = "GLOB")]
    pub exclude: Vec<String>,

    /// Show filename with output
    #[arg(short = 'H', long = "with-filename")]
    pub with_filename: bool,

    /// Don't show filename with output
    #[arg(long = "no-filename")]
    pub no_filename: bool,

    /// Group output by filename
    #[arg(long = "heading")]
    pub heading: bool,

    /// Stop after NUM matches
    #[arg(short = 'm', long = "max-count", value_name = "NUM")]
    pub max_count: Option<usize>,

    /// Number of threads (default: number of CPUs)
    #[arg(short = 'j', long = "jobs", value_name = "NUM")]
    pub jobs: Option<usize>,

    /// Search binary files
    #[arg(long = "binary")]
    pub binary: bool,

    /// When to use colors (auto, always, never)
    #[arg(long = "color", value_name = "WHEN")]
    pub color: Option<String>,

    /// Disable colors
    #[arg(long = "no-color")]
    pub no_color: bool,
}

impl Cli {
    /// Convert CLI arguments to Config
    pub fn to_config(self) -> Result<Config, String> {
        let mut config = Config::default();

        // Collect patterns
        if let Some(pattern) = self.pattern {
            config.patterns.push(pattern);
        }
        config.patterns.extend(self.patterns);

        if config.patterns.is_empty() {
            return Err("No pattern specified. Use PATTERN or -e PATTERN.".to_string());
        }

        // Files
        config.files = self.files;

        // Pattern matching options
        config.case_insensitive = self.ignore_case;
        config.whole_words = self.word_regexp;
        config.invert_match = self.invert_match;

        // Display options
        config.show_line_numbers = self.line_number;
        config.only_matching = self.only_matching;
        config.count_only = self.count;

        // Context
        if let Some(context) = self.context {
            config.context_before = context;
            config.context_after = context;
        } else {
            config.context_before = self.before_context;
            config.context_after = self.after_context;
        }

        // File handling
        config.recursive = self.recursive;
        config.follow_symlinks = self.follow;
        config.hidden = self.hidden;
        config.max_depth = self.max_depth;

        // File filtering
        config.include_patterns = self.include;
        config.exclude_patterns = self.exclude;

        // Output
        config.with_filename = self.with_filename;
        config.no_filename = self.no_filename;
        config.heading_mode = self.heading;

        // Performance
        config.max_count = self.max_count;
        config.jobs = self.jobs;
        config.binary = self.binary;

        // Colors
        config.no_color = self.no_color;
        if let Some(color_when) = self.color {
            config.color_mode = match color_when.to_lowercase().as_str() {
                "always" => ColorMode::Always,
                "never" => ColorMode::Never,
                "auto" => ColorMode::Auto,
                _ => return Err(format!("Invalid --color value: {}. Use auto, always, or never.", color_when)),
            };
        }

        // Auto-detect filename display
        config.auto_detect_filename_display();

        Ok(config)
    }
}
