use jarvis_common::config::WakeWordConfig;
use strsim::levenshtein;

/// Wake word detector with fuzzy matching.
///
/// Supports multiple variants and Levenshtein distance matching
/// to handle accent variations (Turkish speakers saying "jarvis").
pub struct WakeWordDetector {
    primary: String,
    variants: Vec<String>,
    fuzzy_enabled: bool,
    max_distance: usize,
    min_word_length: usize,
}

impl WakeWordDetector {
    pub fn new(config: &WakeWordConfig) -> Self {
        Self {
            primary: config.primary.to_lowercase(),
            variants: config
                .variants
                .iter()
                .map(|v| v.to_lowercase())
                .collect(),
            fuzzy_enabled: config.fuzzy_match.enabled,
            max_distance: config.fuzzy_match.max_distance as usize,
            min_word_length: config.fuzzy_match.min_word_length as usize,
        }
    }

    /// Check if the given text contains the wake word.
    pub fn detect(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();

        for word in &words {
            // Exact match against primary
            if *word == self.primary {
                return true;
            }

            // Exact match against variants
            if self.variants.iter().any(|v| v == word) {
                return true;
            }

            // Fuzzy match
            if self.fuzzy_enabled && word.len() >= self.min_word_length {
                let dist = levenshtein(word, &self.primary);
                if dist <= self.max_distance {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_common::config::{FuzzyMatchConfig, WakeWordConfig};

    fn test_config() -> WakeWordConfig {
        WakeWordConfig {
            primary: "jarvis".into(),
            variants: vec!["carvis".into(), "garvis".into(), "jervis".into()],
            fuzzy_match: FuzzyMatchConfig {
                enabled: true,
                max_distance: 3,
                min_word_length: 4,
            },
        }
    }

    #[test]
    fn exact_match() {
        let det = WakeWordDetector::new(&test_config());
        assert!(det.detect("Hey Jarvis"));
        assert!(det.detect("jarvis ne yapıyorsun"));
    }

    #[test]
    fn variant_match() {
        let det = WakeWordDetector::new(&test_config());
        assert!(det.detect("carvis bana yardım et"));
        assert!(det.detect("garvis merhaba"));
    }

    #[test]
    fn fuzzy_match() {
        let det = WakeWordDetector::new(&test_config());
        assert!(det.detect("jarvıs açıl")); // Turkish ı
        assert!(det.detect("jarbis nasılsın")); // typo
    }

    #[test]
    fn no_match() {
        let det = WakeWordDetector::new(&test_config());
        assert!(!det.detect("merhaba dünya"));
        assert!(!det.detect("bugün hava güzel"));
    }
}
