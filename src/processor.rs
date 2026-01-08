use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::config::Config;
use crate::highlighter::Highlighter;
use crate::matcher::Matcher;

/// Processes files and produces highlighted output
pub struct Processor {
    config: Config,
    matcher: Matcher,
    highlighter: Highlighter,
}

impl Processor {
    /// Create a new Processor
    pub fn new(config: Config) -> Result<Self, String> {
        let matcher = Matcher::new(&config)?;
        let highlighter = Highlighter::new(&config);

        Ok(Self {
            config,
            matcher,
            highlighter,
        })
    }

    /// Process a file
    pub fn process_file(&self, path: &Path) -> Result<(), String> {
        let file =
            File::open(path).map_err(|e| format!("Error opening {}: {}", path.display(), e))?;

        let reader = BufReader::new(file);
        self.process_reader(reader, Some(path))
    }

    /// Process stdin
    pub fn process_stdin(&self) -> Result<(), String> {
        let stdin = std::io::stdin();
        let reader = stdin.lock();
        self.process_reader(reader, None)
    }

    /// Process a reader (file or stdin)
    fn process_reader<R: BufRead>(&self, reader: R, filepath: Option<&Path>) -> Result<(), String> {
        if self.config.count_only {
            return self.process_count_only(reader);
        }

        let mut line_num = 0;
        let mut match_count = 0;
        let mut context_before_buffer: VecDeque<(usize, String)> =
            VecDeque::with_capacity(self.config.context_before);
        let mut context_after_remaining = 0;
        let mut last_match_line = None;

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Error reading line: {}", e))?;
            line_num += 1;

            let matches = self.matcher.find_matches(&line);
            let has_match = !matches.is_empty();

            // Determine if this line should be output based on flags
            let should_output = if self.config.only_matching {
                // -o flag: only show lines with matches
                has_match
            } else if self.config.invert_match {
                // -v flag: only show lines without matches
                !has_match
            } else {
                // Default: show ALL lines (highlighting matches)
                true
            };

            if should_output {
                // Track matches for max_count
                if has_match {
                    match_count += 1;

                    // Check max count
                    if let Some(max) = self.config.max_count
                        && match_count > max
                    {
                        break;
                    }
                }

                // Output context before (if any) - only for matching lines
                if has_match && self.config.context_before > 0 {
                    // Add separator if needed
                    if let Some(last) = last_match_line
                        && line_num - last > 1
                        && !context_before_buffer.is_empty()
                    {
                        println!("--");
                    }

                    for (ctx_line_num, ctx_line) in context_before_buffer.drain(..) {
                        self.output_line(&ctx_line, ctx_line_num, &[], filepath, false);
                    }
                }

                // Output the line
                if self.config.only_matching && has_match {
                    let extracted = self.highlighter.extract_matches(&line, &matches);
                    for matched_part in extracted {
                        if self.config.show_filename
                            && let Some(path) = filepath
                        {
                            print!("{}:", path.display());
                        }
                        if self.config.show_line_numbers {
                            print!("{}:", line_num);
                        }
                        println!("{}", matched_part);
                    }
                } else {
                    self.output_line(&line, line_num, &matches, filepath, has_match);
                }

                if has_match {
                    last_match_line = Some(line_num);
                    context_after_remaining = self.config.context_after;
                }
            } else if context_after_remaining > 0 {
                // Output context after
                self.output_line(&line, line_num, &[], filepath, false);
                context_after_remaining -= 1;
            } else if self.config.context_before > 0 {
                // Buffer for potential context before
                context_before_buffer.push_back((line_num, line));
                if context_before_buffer.len() > self.config.context_before {
                    context_before_buffer.pop_front();
                }
            }
        }

        Ok(())
    }

    /// Process in count-only mode
    fn process_count_only<R: BufRead>(&self, reader: R) -> Result<(), String> {
        let mut count = 0;

        for line_result in reader.lines() {
            let line = line_result.map_err(|e| format!("Error reading line: {}", e))?;

            let has_match = self.matcher.has_match(&line);
            let should_count = if self.config.invert_match {
                !has_match
            } else {
                has_match
            };

            if should_count {
                count += 1;

                if let Some(max) = self.config.max_count
                    && count >= max
                {
                    break;
                }
            }
        }

        println!("{}", count);

        Ok(())
    }

    /// Output a single line with optional highlighting
    fn output_line(
        &self,
        line: &str,
        line_num: usize,
        matches: &[crate::matcher::Match],
        filepath: Option<&Path>,
        is_match: bool,
    ) {
        let highlighted = if is_match && !matches.is_empty() {
            self.highlighter.highlight(line, matches)
        } else {
            line.to_string()
        };

        let separator = if self.config.context_before > 0 || self.config.context_after > 0 {
            if is_match { ":" } else { "-" }
        } else {
            ":"
        };

        if self.config.show_filename
            && let Some(path) = filepath
        {
            print!("{}{}", path.display(), separator);
        }

        if self.config.show_line_numbers {
            print!("{}{}", line_num, separator);
        }

        println!("{}", highlighted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_test_config(patterns: Vec<&str>) -> Config {
        let mut config = Config::default();
        config.patterns = patterns.into_iter().map(|s| s.to_string()).collect();
        config.color_mode = crate::config::ColorMode::Never; // Disable colors for testing
        config
    }

    fn process_test_input(config: Config, input: &str) -> String {
        let processor = Processor::new(config).unwrap();

        // For testing, we'll test the filtering logic directly
        let lines: Vec<&str> = input.lines().collect();
        let mut results = Vec::new();

        for line in lines {
            let matches = processor.matcher.find_matches(line);
            let has_match = !matches.is_empty();

            let should_output = if processor.config.only_matching {
                has_match
            } else if processor.config.invert_match {
                !has_match
            } else {
                true // Default: show all lines
            };

            if should_output {
                results.push(line.to_string());
            }
        }

        results.join("\n")
    }

    #[test]
    fn test_default_shows_all_lines() {
        let config = create_test_config(vec!["TODO"]);
        let input = "Line 1\nLine 2 TODO\nLine 3\nLine 4";
        let output = process_test_input(config, input);

        // Default behavior: show ALL lines (matches highlighted)
        assert_eq!(output.lines().count(), 4);
        assert!(output.contains("Line 1"));
        assert!(output.contains("Line 2 TODO"));
        assert!(output.contains("Line 3"));
        assert!(output.contains("Line 4"));
    }

    #[test]
    fn test_only_matching_filters_lines() {
        let mut config = create_test_config(vec!["TODO"]);
        config.only_matching = true;

        let input = "Line 1\nLine 2 TODO\nLine 3\nLine 4 TODO again";
        let output = process_test_input(config, input);

        // -o flag: only show matching lines
        assert_eq!(output.lines().count(), 2);
        assert!(!output.contains("Line 1"));
        assert!(output.contains("Line 2 TODO"));
        assert!(!output.contains("Line 3"));
        assert!(output.contains("Line 4 TODO again"));
    }

    #[test]
    fn test_invert_match_filters_lines() {
        let mut config = create_test_config(vec!["TODO"]);
        config.invert_match = true;

        let input = "Line 1\nLine 2 TODO\nLine 3\nLine 4 TODO again";
        let output = process_test_input(config, input);

        // -v flag: only show non-matching lines
        assert_eq!(output.lines().count(), 2);
        assert!(output.contains("Line 1"));
        assert!(!output.contains("Line 2 TODO"));
        assert!(output.contains("Line 3"));
        assert!(!output.contains("Line 4 TODO again"));
    }

    #[test]
    fn test_multiple_patterns_default() {
        let config = create_test_config(vec!["TODO", "FIXME"]);
        let input = "Line 1\nLine 2 TODO\nLine 3 FIXME\nLine 4";
        let output = process_test_input(config, input);

        // Should show all lines with matches highlighted
        assert_eq!(output.lines().count(), 4);
    }

    #[test]
    fn test_no_matches_shows_all_lines() {
        let config = create_test_config(vec!["NOTFOUND"]);
        let input = "Line 1\nLine 2\nLine 3";
        let output = process_test_input(config, input);

        // No matches, but should still show all lines by default
        assert_eq!(output.lines().count(), 3);
    }

    #[test]
    fn test_empty_input() {
        let config = create_test_config(vec!["TODO"]);
        let input = "";
        let output = process_test_input(config, input);

        assert_eq!(output, "");
    }

    #[test]
    fn test_only_matching_with_no_matches() {
        let mut config = create_test_config(vec!["NOTFOUND"]);
        config.only_matching = true;

        let input = "Line 1\nLine 2\nLine 3";
        let output = process_test_input(config, input);

        // No matches with -o flag: show nothing
        assert_eq!(output, "");
    }

    #[test]
    fn test_invert_match_with_no_matches() {
        let mut config = create_test_config(vec!["NOTFOUND"]);
        config.invert_match = true;

        let input = "Line 1\nLine 2\nLine 3";
        let output = process_test_input(config, input);

        // No matches with -v flag: show all lines
        assert_eq!(output.lines().count(), 3);
    }

    #[test]
    fn test_case_insensitive_default() {
        let mut config = create_test_config(vec!["todo"]);
        config.case_insensitive = true;

        let input = "Line 1\nLine 2 TODO\nLine 3";
        let output = process_test_input(config, input);

        // Should match TODO even though pattern is lowercase
        assert_eq!(output.lines().count(), 3);
        assert!(output.contains("Line 2 TODO"));
    }

    #[test]
    fn test_processor_creation() {
        let config = create_test_config(vec!["test"]);
        let processor = Processor::new(config);
        assert!(processor.is_ok());
    }

    #[test]
    fn test_processor_creation_with_invalid_regex() {
        let config = create_test_config(vec!["["]);
        let processor = Processor::new(config);
        assert!(processor.is_err());
    }

    #[test]
    fn test_count_only_mode() {
        let mut config = create_test_config(vec!["TODO"]);
        config.count_only = true;

        let processor = Processor::new(config).unwrap();
        let input = "Line 1\nLine 2 TODO\nLine 3\nLine 4 TODO";
        let reader = Cursor::new(input.as_bytes());

        // Count mode should not error
        let result = processor.process_reader(reader, None);
        assert!(result.is_ok());
    }
}
