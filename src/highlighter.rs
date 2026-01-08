use colored::{Color, Colorize};

use crate::config::Config;
use crate::matcher::Match;

/// Handles text highlighting with colors
pub struct Highlighter {
    use_colors: bool,
    colors: Vec<Color>,
}

impl Highlighter {
    /// Create a new Highlighter from config
    pub fn new(config: &Config) -> Self {
        let colors = vec![
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::BrightRed,
            Color::BrightGreen,
            Color::BrightYellow,
            Color::BrightBlue,
            Color::BrightMagenta,
            Color::BrightCyan,
        ];

        Self {
            use_colors: config.should_use_color(),
            colors,
        }
    }

    /// Get color for a pattern index
    fn get_color(&self, pattern_index: usize) -> Color {
        self.colors[pattern_index % self.colors.len()]
    }

    /// Highlight all matches in a line
    pub fn highlight(&self, text: &str, matches: &[Match]) -> String {
        if !self.use_colors || matches.is_empty() {
            return text.to_string();
        }

        let mut result = String::new();
        let mut last_pos = 0;

        for m in matches {
            // Skip overlapping matches (keep the first one)
            if m.start < last_pos {
                continue;
            }

            // Add text before the match
            result.push_str(&text[last_pos..m.start]);

            // Add highlighted match
            let color = self.get_color(m.pattern_index);
            result.push_str(&text[m.start..m.end].color(color).bold().to_string());

            last_pos = m.end;
        }

        // Add remaining text after last match
        result.push_str(&text[last_pos..]);

        result
    }

    /// Extract only the matching parts (for -o flag)
    pub fn extract_matches(&self, text: &str, matches: &[Match]) -> Vec<String> {
        let mut results = Vec::new();
        let mut last_end = 0;

        for m in matches {
            // Skip overlapping matches
            if m.start < last_end {
                continue;
            }

            let matched_text = &text[m.start..m.end];

            if self.use_colors {
                let color = self.get_color(m.pattern_index);
                results.push(matched_text.color(color).bold().to_string());
            } else {
                results.push(matched_text.to_string());
            }

            last_end = m.end;
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_no_colors() {
        let config = Config {
            no_color: true,
            ..Default::default()
        };

        let highlighter = Highlighter::new(&config);
        let matches = vec![Match {
            start: 0,
            end: 5,
            pattern_index: 0,
        }];

        let result = highlighter.highlight("hello world", &matches);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_extract_matches() {
        let config = Config {
            no_color: true,
            ..Default::default()
        };

        let highlighter = Highlighter::new(&config);
        let matches = vec![
            Match {
                start: 0,
                end: 5,
                pattern_index: 0,
            },
            Match {
                start: 6,
                end: 11,
                pattern_index: 1,
            },
        ];

        let result = highlighter.extract_matches("hello world", &matches);
        assert_eq!(result, vec!["hello", "world"]);
    }
}
