use regex::{Regex, RegexBuilder};

use crate::config::Config;

/// Holds compiled regex patterns for matching
pub struct Matcher {
    regexes: Vec<Regex>,
}

/// Represents a single match within a line
#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub pattern_index: usize,
}

impl Matcher {
    /// Create a new Matcher from config
    pub fn new(config: &Config) -> Result<Self, String> {
        let mut regexes = Vec::new();

        for pattern in &config.patterns {
            // Wrap pattern with word boundaries if needed
            let final_pattern = if config.whole_words {
                format!(r"\b{}\b", pattern)
            } else {
                pattern.clone()
            };

            let regex = RegexBuilder::new(&final_pattern)
                .case_insensitive(config.case_insensitive)
                .build()
                .map_err(|e| format!("Invalid regex pattern '{}': {}", pattern, e))?;

            regexes.push(regex);
        }

        Ok(Self { regexes })
    }

    /// Find all matches in a line
    pub fn find_matches(&self, text: &str) -> Vec<Match> {
        let mut all_matches = Vec::new();

        for (pattern_index, regex) in self.regexes.iter().enumerate() {
            for mat in regex.find_iter(text) {
                all_matches.push(Match {
                    start: mat.start(),
                    end: mat.end(),
                    pattern_index,
                });
            }
        }

        // Sort by start position
        all_matches.sort_by(|a, b| a.start.cmp(&b.start));

        all_matches
    }

    /// Check if a line has any matches
    pub fn has_match(&self, text: &str) -> bool {
        self.regexes.iter().any(|regex| regex.is_match(text))
    }

    /// Get the number of patterns
    pub fn pattern_count(&self) -> usize {
        self.regexes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_matching() {
        let mut config = Config::default();
        config.patterns = vec!["hello".to_string()];

        let matcher = Matcher::new(&config).unwrap();
        let matches = matcher.find_matches("hello world");

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[0].end, 5);
    }

    #[test]
    fn test_case_insensitive() {
        let mut config = Config::default();
        config.patterns = vec!["hello".to_string()];
        config.case_insensitive = true;

        let matcher = Matcher::new(&config).unwrap();
        let matches = matcher.find_matches("HELLO world");

        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_multiple_patterns() {
        let mut config = Config::default();
        config.patterns = vec!["hello".to_string(), "world".to_string()];

        let matcher = Matcher::new(&config).unwrap();
        let matches = matcher.find_matches("hello world");

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].pattern_index, 0);
        assert_eq!(matches[1].pattern_index, 1);
    }

    #[test]
    fn test_whole_words() {
        let mut config = Config::default();
        config.patterns = vec!["cat".to_string()];
        config.whole_words = true;

        let matcher = Matcher::new(&config).unwrap();

        assert!(matcher.has_match("cat dog"));
        assert!(!matcher.has_match("cats"));
        assert!(!matcher.has_match("bobcat"));
    }
}
