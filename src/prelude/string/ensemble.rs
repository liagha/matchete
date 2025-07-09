use {
    super::{
        exact::{Exact, Relaxed},
        fuzzy::{Jaro, Cosine, Levenshtein},
        structural::{Prefix, Suffix, NGram, Contains},
        lexical::{Tokens, Initials, Words},
        proximity::{Keyboard, Fuzzy},
        phonetic::Phonetic,
    },
    crate::{
        assessor::{
            Resembler, Resemblance, Dimension, Blend
        },
    }
};

#[derive(Debug)]
pub struct Aligner {
    blend: Blend<String, String, ()>,
}

impl Default for Aligner {
    fn default() -> Self {
        let dimensions = vec![
            Dimension::new(Jaro::default(), 0.2),
            Dimension::new(Cosine::default(), 0.15),
            Dimension::new(Exact, 0.1),
            Dimension::new(Relaxed, 0.1),
            Dimension::new(Prefix, 0.1),
            Dimension::new(Suffix, 0.05),
            Dimension::new(Contains, 0.05),
            Dimension::new(Levenshtein, 0.1),
            Dimension::new(Tokens::default(), 0.1),
            Dimension::new(Initials::default(), 0.05),
            Dimension::new(Keyboard::default(), 0.05),
            Dimension::new(Fuzzy::default(), 0.1),
            Dimension::new(Phonetic::default(), 0.05),
            Dimension::new(NGram::default(), 0.05),
            Dimension::new(Words::default(), 0.1),
        ];
        Self {
            blend: Blend::weighted(dimensions),
        }
    }
}

impl Resembler<String, String, ()> for Aligner {
    fn resemblance(&self, query: &String, candidate: &String) -> Result<Resemblance, ()> {
        self.blend.resemblance(query, candidate)
    }
}