use glob::Pattern;
use ignore::WalkBuilder;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::config::Config;

/// Discovers files to search based on config
pub struct FileDiscovery<'a> {
    config: &'a Config,
}

impl<'a> FileDiscovery<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Discover all files to search
    pub fn discover(&self) -> Result<Vec<PathBuf>, String> {
        if self.config.files.is_empty() {
            // No files specified, use stdin
            return Ok(Vec::new());
        }

        let mut result = Vec::new();

        for path in &self.config.files {
            if path.is_file() {
                // Single file
                if self.should_include_file(path) {
                    result.push(path.clone());
                }
            } else if path.is_dir() {
                // Directory
                if self.config.recursive {
                    result.extend(self.walk_directory(path)?);
                } else {
                    return Err(format!(
                        "{} is a directory (use -r for recursive search)",
                        path.display()
                    ));
                }
            } else {
                return Err(format!("{}: No such file or directory", path.display()));
            }
        }

        Ok(result)
    }

    /// Walk a directory recursively
    fn walk_directory(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let mut builder = WalkBuilder::new(path);

        builder
            .follow_links(self.config.follow_symlinks)
            .hidden(!self.config.hidden)
            .git_ignore(true) // Respect .gitignore by default
            .git_global(true)
            .git_exclude(true);

        if let Some(max_depth) = self.config.max_depth {
            builder.max_depth(Some(max_depth));
        }

        let mut results = Vec::new();

        for entry in builder.build() {
            let entry = entry.map_err(|e| format!("Error walking directory: {}", e))?;

            let path = entry.path();

            // Skip directories
            if !path.is_file() {
                continue;
            }

            // Check if we should include this file
            if !self.should_include_file(path) {
                continue;
            }

            // Skip binary files unless --binary is set
            if !self.config.binary && is_binary(path)? {
                continue;
            }

            results.push(path.to_path_buf());
        }

        Ok(results)
    }

    /// Check if a file should be included based on include/exclude patterns
    fn should_include_file(&self, path: &Path) -> bool {
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        // Check exclude patterns first
        for exclude in &self.config.exclude_patterns {
            if let Ok(pattern) = Pattern::new(exclude) {
                if pattern.matches(filename) || pattern.matches(path.to_str().unwrap_or("")) {
                    return false;
                }
            }
        }

        // If there are include patterns, file must match at least one
        if !self.config.include_patterns.is_empty() {
            for include in &self.config.include_patterns {
                if let Ok(pattern) = Pattern::new(include) {
                    if pattern.matches(filename) || pattern.matches(path.to_str().unwrap_or("")) {
                        return true;
                    }
                }
            }
            return false;
        }

        true
    }
}

/// Check if a file appears to be binary by reading the first 8KB
fn is_binary(path: &Path) -> Result<bool, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("Error opening {}: {}", path.display(), e))?;

    let mut buffer = vec![0; 8192];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|e| format!("Error reading {}: {}", path.display(), e))?;

    // Check for null bytes (common indicator of binary files)
    Ok(buffer[..bytes_read].contains(&0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_pattern() {
        let mut config = Config::default();
        config.include_patterns = vec!["*.rs".to_string()];

        let discovery = FileDiscovery::new(&config);

        assert!(discovery.should_include_file(Path::new("test.rs")));
        assert!(!discovery.should_include_file(Path::new("test.txt")));
    }

    #[test]
    fn test_exclude_pattern() {
        let mut config = Config::default();
        config.exclude_patterns = vec!["*.min.js".to_string()];

        let discovery = FileDiscovery::new(&config);

        assert!(!discovery.should_include_file(Path::new("app.min.js")));
        assert!(discovery.should_include_file(Path::new("app.js")));
    }
}
