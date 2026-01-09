use clap::Parser;
use std::path::PathBuf;

use crate::config::{ColorMode, Config};

/// Crepe - A grep-like tool that highlights matches instead of filtering lines
#[derive(Parser, Debug)]
#[command(name = "crepe")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Pattern to search for (regex) - use -e for each pattern
    #[arg(short = 'e', long = "regexp", value_name = "PATTERN", required = true)]
    pub patterns: Vec<String>,

    /// Files to search (or stdin if not specified)
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

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
    #[arg(
        short = 'B',
        long = "before-context",
        value_name = "NUM",
        default_value = "0"
    )]
    pub before_context: usize,

    /// Lines of context after matches
    #[arg(
        short = 'A',
        long = "after-context",
        value_name = "NUM",
        default_value = "0"
    )]
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
    pub fn into_config(self) -> Result<Config, String> {
        // Parse color mode
        let color_mode = if let Some(color_when) = self.color {
            match color_when.to_lowercase().as_str() {
                "always" => ColorMode::Always,
                "never" => ColorMode::Never,
                "auto" => ColorMode::Auto,
                _ => {
                    return Err(format!(
                        "Invalid --color value: {}. Use auto, always, or never.",
                        color_when
                    ));
                }
            }
        } else {
            ColorMode::Auto
        };

        // Determine context values
        let (context_before, context_after) = if let Some(context) = self.context {
            (context, context)
        } else {
            (self.before_context, self.after_context)
        };

        // Create config with all fields initialized
        let mut config = Config {
            patterns: self.patterns,
            files: self.files,
            case_insensitive: self.ignore_case,
            whole_words: self.word_regexp,
            invert_match: self.invert_match,
            show_line_numbers: self.line_number,
            only_matching: self.only_matching,
            count_only: self.count,
            context_before,
            context_after,
            recursive: self.recursive,
            follow_symlinks: self.follow,
            hidden: self.hidden,
            max_depth: self.max_depth,
            include_patterns: self.include,
            exclude_patterns: self.exclude,
            with_filename: self.with_filename,
            no_filename: self.no_filename,
            max_count: self.max_count,
            jobs: self.jobs,
            binary: self.binary,
            no_color: self.no_color,
            color_mode,
            ..Default::default()
        };

        // Auto-detect filename display
        config.auto_detect_filename_display();

        Ok(config)
    }
}
