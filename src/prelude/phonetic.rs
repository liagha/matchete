use crate::Scorer;

/// Soundex phonetic encoding for names with improved handling of edge cases
/// and performance optimizations
#[derive(Debug)]
pub struct SoundexScorer {
    /// Maximum length to consider for calculating partial matches
    max_compare_length: usize,
    /// Enables special handling for non-English phonetic patterns
    international_mode: bool,
}

impl Default for SoundexScorer {
    fn default() -> Self {
        SoundexScorer {
            max_compare_length: 4,
            international_mode: false,
        }
    }
}

impl SoundexScorer {
    /// Creates a new SoundexScorer with custom settings
    pub fn new(max_compare_length: usize, international_mode: bool) -> Self {
        SoundexScorer {
            max_compare_length: max_compare_length.max(1).min(10),
            international_mode,
        }
    }

    /// Generates Soundex code for a string
    pub fn soundex(&self, s: &str) -> String {
        if s.is_empty() {
            return "0000".to_string();
        }

        let chars: Vec<char> = s.to_uppercase().chars().collect();

        let first_char = chars.iter()
            .find(|&&c| c.is_ascii_alphabetic())
            .copied()
            .unwrap_or('0');

        if first_char == '0' {
            return "0000".to_string();
        }

        let mut code = Vec::new();
        let mut last_digit = '0';

        for &c in &chars {
            let mut digit = match c {
                'B' | 'F' | 'P' | 'V' => '1',
                'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
                'D' | 'T' => '3',
                'L' => '4',
                'M' | 'N' => '5',
                'R' => '6',
                _ => '0',
            };

            // International mode handling
            if self.international_mode {
                // Additional international phonetic patterns
                digit = match c {
                    'Ñ' | 'Ń' => '5',
                    'Ç' => '2',
                    'Ø' | 'Ö' => '0',
                    'Æ' => '0',
                    'Ł' => '4',
                    _ => digit,
                };
            }

            // Skip vowels and 'H', 'W', 'Y'
            if digit == '0' {
                continue;
            }

            // Skip repeating consonant sounds
            if digit != last_digit {
                code.push(digit);
                last_digit = digit;
            }
        }

        // Build result string
        let mut result = first_char.to_string();

        // Append code digits
        result.push_str(&code.into_iter().collect::<String>());

        // Ensure we have a correctly sized code (standard is 4 characters)
        if result.len() < self.max_compare_length + 1 {
            result.push_str(&"0".repeat(self.max_compare_length + 1 - result.len()));
        } else {
            result.truncate(self.max_compare_length + 1);
        }

        result
    }

    /// Calculates partial matching score for two Soundex codes
    fn partial_match_score(&self, code1: &str, code2: &str) -> f64 {
        let code1_chars: Vec<char> = code1.chars().collect();
        let code2_chars: Vec<char> = code2.chars().collect();

        // Count matching positions
        let matching = code1_chars.iter().zip(code2_chars.iter())
            .filter(|(a, b)| a == b)
            .count();

        // First character match is weighted more heavily
        let first_char_bonus = if !code1_chars.is_empty() && !code2_chars.is_empty()
            && code1_chars[0] == code2_chars[0] { 0.1 } else { 0.0 };

        (matching as f64 / self.max_compare_length as f64) + first_char_bonus
    }
}

impl Scorer<String, String> for SoundexScorer {
    fn score(&self, query: &String, candidate: &String) -> f64 {
        // Handle empty strings
        if query.is_empty() && candidate.is_empty() {
            return 1.0;
        }

        if query.is_empty() || candidate.is_empty() {
            return 0.0;
        }

        let query_soundex = self.soundex(query);
        let item_soundex = self.soundex(candidate);

        if query_soundex == item_soundex {
            return 1.0;
        }

        // Calculate partial match score
        self.partial_match_score(&query_soundex, &item_soundex)
    }

    fn exact(&self, query: &String, candidate: &String) -> bool {
        // Handle empty strings
        if query.is_empty() && candidate.is_empty() {
            return true;
        }

        if query.is_empty() || candidate.is_empty() {
            return false;
        }

        let query_soundex = self.soundex(query);
        let item_soundex = self.soundex(candidate);

        query_soundex == item_soundex
    }
}

// Add support for string references
impl Scorer<&str, String> for SoundexScorer {
    fn score(&self, query: &&str, candidate: &String) -> f64 {
        let query_str = query.to_string();
        self.score(&query_str, candidate)
    }

    fn exact(&self, query: &&str, candidate: &String) -> bool {
        let query_str = query.to_string();
        self.exact(&query_str, candidate)
    }
}

impl Scorer<String, &str> for SoundexScorer {
    fn score(&self, query: &String, candidate: &&str) -> f64 {
        let item_str = candidate.to_string();
        self.score(query, &item_str)
    }

    fn exact(&self, query: &String, candidate: &&str) -> bool {
        let item_str = candidate.to_string();
        self.exact(query, &item_str)
    }
}

impl Scorer<&str, &str> for SoundexScorer {
    fn score(&self, query: &&str, candidate: &&str) -> f64 {
        let query_str = query.to_string();
        let item_str = candidate.to_string();
        self.score(&query_str, &item_str)
    }

    fn exact(&self, query: &&str, candidate: &&str) -> bool {
        let query_str = query.to_string();
        let item_str = candidate.to_string();
        self.exact(&query_str, &item_str)
    }
}