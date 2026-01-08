use std::path::PathBuf;

/// Configuration for the crepe text highlighter
#[derive(Debug, Clone)]
pub struct Config {
    // Pattern matching
    pub patterns: Vec<String>,
    pub case_insensitive: bool,
    pub whole_words: bool,
    pub invert_match: bool,

    // Display options
    pub show_line_numbers: bool,
    pub only_matching: bool,
    pub count_only: bool,
    pub no_color: bool,
    pub color_mode: ColorMode,

    // Context
    pub context_before: usize,
    pub context_after: usize,

    // File handling
    pub files: Vec<PathBuf>,
    pub recursive: bool,
    pub follow_symlinks: bool,
    pub hidden: bool,
    pub max_depth: Option<usize>,

    // File filtering
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,

    // Output options
    pub show_filename: bool,
    pub with_filename: bool,
    pub no_filename: bool,
    pub heading_mode: bool,

    // Performance
    pub max_count: Option<usize>,
    pub jobs: Option<usize>,
    pub binary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            case_insensitive: false,
            whole_words: false,
            invert_match: false,
            show_line_numbers: false,
            only_matching: false,
            count_only: false,
            no_color: false,
            color_mode: ColorMode::Auto,
            context_before: 0,
            context_after: 0,
            files: Vec::new(),
            recursive: false,
            follow_symlinks: false,
            hidden: false,
            max_depth: None,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            show_filename: false,
            with_filename: false,
            no_filename: false,
            heading_mode: false,
            max_count: None,
            jobs: None,
            binary: false,
        }
    }
}

impl Config {
    /// Auto-detect whether to show filenames based on file count
    pub fn auto_detect_filename_display(&mut self) {
        if !self.with_filename && !self.no_filename {
            self.show_filename = self.files.len() > 1 || self.recursive;
        } else if self.with_filename {
            self.show_filename = true;
        }
    }

    /// Check if colors should be enabled
    pub fn should_use_color(&self) -> bool {
        if self.no_color {
            return false;
        }

        match self.color_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => {
                // Check if stdout is a TTY
                use std::io::IsTerminal;
                std::io::stdout().is_terminal()
            }
        }
    }
}
