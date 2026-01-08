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
        let file = File::open(path)
            .map_err(|e| format!("Error opening {}: {}", path.display(), e))?;

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
    fn process_reader<R: BufRead>(
        &self,
        reader: R,
        filepath: Option<&Path>,
    ) -> Result<(), String> {
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

            // Apply invert match logic
            let should_output = if self.config.invert_match {
                !has_match
            } else {
                has_match
            };

            if should_output {
                match_count += 1;

                // Check max count
                if let Some(max) = self.config.max_count
                    && match_count > max {
                        break;
                    }

                // Output context before (if any)
                if self.config.context_before > 0 {
                    // Add separator if needed
                    if let Some(last) = last_match_line
                        && line_num - last > 1 && !context_before_buffer.is_empty() {
                            println!("--");
                            }

                    for (ctx_line_num, ctx_line) in context_before_buffer.drain(..) {
                        self.output_line(&ctx_line, ctx_line_num, &[], filepath, false);
                    }
                }

                // Output the matching line
                if self.config.only_matching {
                    let extracted = self.highlighter.extract_matches(&line, &matches);
                    for matched_part in extracted {
                        if self.config.show_filename
                            && let Some(path) = filepath {
                                print!("{}:", path.display());
                            }
                        if self.config.show_line_numbers {
                            print!("{}:", line_num);
                        }
                        println!("{}", matched_part);
                    }
                } else {
                    self.output_line(&line, line_num, &matches, filepath, true);
                }

                last_match_line = Some(line_num);
                context_after_remaining = self.config.context_after;
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
                    && count >= max {
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
            && let Some(path) = filepath {
                print!("{}{}", path.display(), separator);
            }

        if self.config.show_line_numbers {
            print!("{}{}", line_num, separator);
        }

        println!("{}", highlighted);
    }
}
