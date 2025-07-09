use crate::assessor::{Resembler, Resemblance};

#[derive(PartialEq)]
pub struct Phonetic {
    mode: PhoneticMode,
}

#[derive(PartialEq)]
pub enum PhoneticMode {
    Soundex,
    DoubleMetaphone,
}

impl Default for Phonetic {
    fn default() -> Self {
        Self { mode: PhoneticMode::Soundex }
    }
}

impl Phonetic {
    pub fn new(mode: PhoneticMode) -> Self {
        Self { mode }
    }

    fn compute_soundex(&self, text: &str) -> String {
        if text.is_empty() { return "0000".to_string(); }

        let mut result = String::new();
        let mut prev_code = 0;
        for (i, c) in text.to_lowercase().chars().enumerate() {
            let code = match c {
                'b' | 'f' | 'p' | 'v' => 1,
                'c' | 'g' | 'j' | 'k' | 'q' | 's' | 'x' | 'z' => 2,
                'd' | 't' => 3,
                'l' => 4,
                'm' | 'n' => 5,
                'r' => 6,
                _ => 0,
            };

            if i == 0 {
                result.push(c.to_ascii_uppercase());
            } else if code != 0 && code != prev_code {
                result.push(char::from_digit(code, 10).unwrap());
            }

            prev_code = code;
            if result.len() >= 4 { break; }
        }

        while result.len() < 4 {
            result.push('0');
        }
        result
    }
}

impl Resembler<String, String, ()> for Phonetic {
    fn resemblance(&mut self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        if query == candidate {
            return Ok(Resemblance::Perfect);
        }

        let result = match self.mode {
            PhoneticMode::Soundex => {
                let query_code = self.compute_soundex(query);
                let candidate_code = self.compute_soundex(candidate);
                if query_code == candidate_code {
                    Resemblance::Partial(0.85)
                } else {
                    let common_prefix_len = query_code.chars().zip(candidate_code.chars())
                        .take_while(|(c1, c2)| c1 == c2)
                        .count();
                    if common_prefix_len > 0 {
                        Resemblance::Partial(0.6 * (common_prefix_len as f64 / 4.0))
                    } else {
                        Resemblance::Disparity
                    }
                }
            }
            PhoneticMode::DoubleMetaphone => {
                if query.to_lowercase() == candidate.to_lowercase() {
                    return Ok(Resemblance::Perfect);
                }
                let query_code = self.compute_soundex(query);
                let candidate_code = self.compute_soundex(candidate);
                if query_code == candidate_code {
                    Resemblance::Partial(0.8)
                } else {
                    Resemblance::Disparity
                }
            }
        };

        Ok(result)
    }
}