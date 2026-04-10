/// A single word with precomputed display metadata.
#[derive(Debug, Clone)]
pub struct Word {
    /// The raw text of the word.
    pub text: String,
    /// Index of the ORP (Optimal Recognition Point) letter within `text`.
    pub orp_index: usize,
    /// Multiplier for base delay based on word length (e.g., 0.85 for short words, 1.6 for long).
    pub base_delay_multiplier: f32,
    /// Additional delay multiplier for trailing punctuation (e.g., +0.4 for comma, +1.0 for period).
    pub punctuation_delay_multiplier: f32,
    /// If true, this "word" represents a paragraph break (blank frame).
    pub is_paragraph_break: bool,
}

/// A sentence is an ordered list of words.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Sentence {
    pub words: Vec<Word>,
}

/// A paragraph is an ordered list of sentences.
#[derive(Debug, Clone)]
pub struct Paragraph {
    pub sentences: Vec<Sentence>,
}

/// A full document: ordered list of paragraphs with metadata.
#[derive(Debug, Clone)]
pub struct Document {
    pub paragraphs: Vec<Paragraph>,
    /// Flat index of all words for O(1) position lookup.
    pub words: Vec<Word>,
    /// Mapping from flat word index to (paragraph_index, sentence_index, word_in_sentence_index).
    pub word_positions: Vec<(usize, usize, usize)>,
    /// Total number of words (excluding paragraph break markers).
    pub total_words: usize,
}

impl Document {
    /// Returns the flat index of the first word in the sentence containing `word_index`.
    pub fn sentence_start_index(&self, word_index: usize) -> usize {
        if word_index >= self.words.len() {
            return self.words.len();
        }
        let (para_idx, sent_idx, _) = self.word_positions[word_index];
        // Walk backwards from word_index to find the first word in this sentence
        for i in (0..=word_index).rev() {
            let (pi, si, _) = self.word_positions[i];
            if pi != para_idx || si != sent_idx {
                return i + 1;
            }
        }
        0
    }

    /// Returns the flat index of the first word in the next sentence after `word_index`.
    /// Returns `words.len()` if at the end.
    pub fn next_sentence_start_index(&self, word_index: usize) -> usize {
        if word_index >= self.words.len() {
            return self.words.len();
        }
        let (para_idx, sent_idx, _) = self.word_positions[word_index];
        // Walk forward to find the first word not in this sentence
        for i in (word_index + 1)..self.words.len() {
            let (pi, si, _) = self.word_positions[i];
            if pi != para_idx || si != sent_idx {
                return i;
            }
        }
        self.words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tokenizer::tokenize;

    #[test]
    fn sentence_start_first_word() {
        let doc = tokenize("Hello world. Goodbye world.");
        assert_eq!(doc.sentence_start_index(0), 0);
    }

    #[test]
    fn sentence_start_mid_sentence() {
        let doc = tokenize("Hello world. Goodbye world.");
        // "world." is index 1 — first sentence starts at 0
        assert_eq!(doc.sentence_start_index(1), 0);
    }

    #[test]
    fn sentence_start_second_sentence() {
        let doc = tokenize("Hello world. Goodbye world.");
        // "Goodbye" is the first word of the second sentence
        let goodbye_idx = 2;
        assert_eq!(doc.sentence_start_index(goodbye_idx), goodbye_idx);
    }

    #[test]
    fn next_sentence_start_from_first_sentence() {
        let doc = tokenize("Hello world. Goodbye world.");
        let next = doc.next_sentence_start_index(0);
        assert_eq!(next, 2); // "Goodbye" starts at index 2
    }

    #[test]
    fn next_sentence_start_at_last_sentence() {
        let doc = tokenize("Hello world. Goodbye world.");
        // "Goodbye" is start of last sentence
        let next = doc.next_sentence_start_index(2);
        assert_eq!(next, doc.words.len());
    }

    #[test]
    fn sentence_start_out_of_bounds() {
        let doc = tokenize("Hello world.");
        assert_eq!(doc.sentence_start_index(999), doc.words.len());
    }

    #[test]
    fn next_sentence_start_out_of_bounds() {
        let doc = tokenize("Hello world.");
        assert_eq!(doc.next_sentence_start_index(999), doc.words.len());
    }
}
