use super::document::{Document, Paragraph, Sentence, Word};
use super::orp;
use super::timing;

/// Parse a plain text string into a Document.
/// Splits on double newlines for paragraphs, sentence-ending punctuation for sentences,
/// and whitespace for words. Computes ORP and timing for each word.
pub fn tokenize(text: &str) -> Document {
    let mut paragraphs = Vec::new();
    let mut flat_words: Vec<Word> = Vec::new();
    let mut word_positions: Vec<(usize, usize, usize)> = Vec::new();
    let mut total_words: usize = 0;

    // Split on double newlines for paragraphs
    let raw_paragraphs: Vec<&str> = split_paragraphs(text);

    for (para_idx, raw_para) in raw_paragraphs.iter().enumerate() {
        // Insert paragraph break marker between paragraphs (not before first)
        if para_idx > 0 {
            let break_word = Word {
                text: String::new(),
                orp_index: 0,
                base_delay_multiplier: 1.0,
                punctuation_delay_multiplier: 2.0,
                is_paragraph_break: true,
            };
            // Paragraph break markers get a position pointing to the previous paragraph's end
            // We use a special sentinel: the break "belongs" to the gap between paragraphs
            // For position mapping, assign it to the upcoming paragraph, sentence 0, word 0
            // but it will be skipped by total_words count
            word_positions.push((para_idx, 0, 0));
            flat_words.push(break_word);
        }

        let sentences = split_sentences(raw_para);
        let mut para_sentences = Vec::new();

        for (sent_idx, raw_sent) in sentences.iter().enumerate() {
            let words_text: Vec<&str> = raw_sent.split_whitespace().collect();
            if words_text.is_empty() {
                continue;
            }

            let mut sentence_words = Vec::new();

            for (word_idx, &word_text) in words_text.iter().enumerate() {
                let word_len = word_text.chars().count();
                let word = Word {
                    text: word_text.to_string(),
                    orp_index: orp::orp_index(word_len),
                    base_delay_multiplier: timing::length_multiplier(word_len),
                    punctuation_delay_multiplier: timing::punctuation_multiplier(word_text),
                    is_paragraph_break: false,
                };

                word_positions.push((para_idx, sent_idx, word_idx));
                flat_words.push(word.clone());
                sentence_words.push(word);
                total_words += 1;
            }

            para_sentences.push(Sentence {
                words: sentence_words,
            });
        }

        paragraphs.push(Paragraph {
            sentences: para_sentences,
        });
    }

    Document {
        paragraphs,
        words: flat_words,
        word_positions,
        total_words,
    }
}

/// Split text into paragraph chunks on double newlines (or more).
fn split_paragraphs(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return result;
    }

    // Split on two or more consecutive newlines (with optional whitespace between)
    let mut start = 0;
    let bytes = trimmed.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for \n\n pattern (possibly with \r and spaces between)
        if bytes[i] == b'\n' {
            let newline_start = i;
            // Count consecutive newlines (allowing \r\n and whitespace-only lines)
            let mut newline_count = 0;
            let mut j = i;
            while j < len {
                // Skip \r
                if bytes[j] == b'\r' {
                    j += 1;
                    continue;
                }
                if bytes[j] == b'\n' {
                    newline_count += 1;
                    j += 1;
                    // After a newline, skip spaces/tabs on the blank line
                    while j < len && (bytes[j] == b' ' || bytes[j] == b'\t') {
                        j += 1;
                    }
                } else {
                    break;
                }
            }

            if newline_count >= 2 {
                let chunk = &trimmed[start..newline_start];
                let chunk = chunk.trim();
                if !chunk.is_empty() {
                    result.push(chunk);
                }
                start = j;
                i = j;
            } else {
                i = j;
            }
        } else {
            i += 1;
        }
    }

    // Last chunk
    let chunk = &trimmed[start..];
    let chunk = chunk.trim();
    if !chunk.is_empty() {
        result.push(chunk);
    }

    result
}

/// Split a paragraph into sentences. A sentence ends with `.`, `!`, or `?`
/// followed by whitespace or end-of-string.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    for i in 0..len {
        current.push(chars[i]);

        let is_sentence_end = matches!(chars[i], '.' | '!' | '?');

        if is_sentence_end {
            // Check if followed by whitespace or end-of-string
            let at_end = i + 1 >= len;
            let followed_by_space = !at_end && chars[i + 1].is_whitespace();

            if at_end || followed_by_space {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current = String::new();
            }
        }
    }

    // Any remaining text that didn't end with sentence punctuation
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    sentences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        let doc = tokenize("");
        assert_eq!(doc.words.len(), 0);
        assert_eq!(doc.total_words, 0);
        assert_eq!(doc.paragraphs.len(), 0);
    }

    #[test]
    fn single_word() {
        let doc = tokenize("Hello");
        assert_eq!(doc.total_words, 1);
        assert_eq!(doc.words.len(), 1);
        assert_eq!(doc.words[0].text, "Hello");
        assert!(!doc.words[0].is_paragraph_break);
    }

    #[test]
    fn single_sentence() {
        let doc = tokenize("Hello world today.");
        assert_eq!(doc.total_words, 3);
        assert_eq!(doc.words.len(), 3);
        assert_eq!(doc.paragraphs.len(), 1);
        assert_eq!(doc.paragraphs[0].sentences.len(), 1);
    }

    #[test]
    fn two_sentences() {
        let doc = tokenize("Hello world. Goodbye world.");
        assert_eq!(doc.total_words, 4);
        assert_eq!(doc.paragraphs.len(), 1);
        assert_eq!(doc.paragraphs[0].sentences.len(), 2);
        assert_eq!(doc.paragraphs[0].sentences[0].words.len(), 2);
        assert_eq!(doc.paragraphs[0].sentences[1].words.len(), 2);
    }

    #[test]
    fn two_paragraphs() {
        let doc = tokenize("First paragraph.\n\nSecond paragraph.");
        assert_eq!(doc.paragraphs.len(), 2);
        // Flat words: "First", "paragraph.", [break], "Second", "paragraph."
        assert_eq!(doc.words.len(), 5);
        assert_eq!(doc.total_words, 4); // break not counted
        assert!(doc.words[2].is_paragraph_break);
        assert_eq!(doc.words[2].text, "");
    }

    #[test]
    fn paragraph_break_has_correct_timing() {
        let doc = tokenize("First.\n\nSecond.");
        let break_word = &doc.words[1]; // "First." is index 0, break is index 1
        assert!(break_word.is_paragraph_break);
        assert_eq!(break_word.punctuation_delay_multiplier, 2.0);
    }

    #[test]
    fn word_positions_mapping() {
        let doc = tokenize("Hello world. Goodbye.");
        // word 0: (0, 0, 0) - para 0, sent 0, word 0
        // word 1: (0, 0, 1) - para 0, sent 0, word 1
        // word 2: (0, 1, 0) - para 0, sent 1, word 0
        assert_eq!(doc.word_positions[0], (0, 0, 0));
        assert_eq!(doc.word_positions[1], (0, 0, 1));
        assert_eq!(doc.word_positions[2], (0, 1, 0));
    }

    #[test]
    fn orp_is_computed() {
        let doc = tokenize("Hi extraordinary");
        // "Hi" has length 2: orp = floor(2*0.25)=0
        assert_eq!(doc.words[0].orp_index, 0);
        // "extraordinary" has length 13: orp = floor(13*0.25)=3
        assert_eq!(doc.words[1].orp_index, 3);
    }

    #[test]
    fn timing_multipliers_computed() {
        let doc = tokenize("Hi hello,");
        // "Hi" - length 2, short word: 0.85, no punctuation: 0.0
        assert_eq!(doc.words[0].base_delay_multiplier, 0.85);
        assert_eq!(doc.words[0].punctuation_delay_multiplier, 0.0);
        // "hello," - length 6, medium: 1.0, comma: 0.4
        assert_eq!(doc.words[1].base_delay_multiplier, 1.0);
        assert_eq!(doc.words[1].punctuation_delay_multiplier, 0.4);
    }

    #[test]
    fn multiple_paragraphs_with_multiple_sentences() {
        let doc = tokenize("A B. C D.\n\nE F. G H.");
        assert_eq!(doc.paragraphs.len(), 2);
        assert_eq!(doc.paragraphs[0].sentences.len(), 2);
        assert_eq!(doc.paragraphs[1].sentences.len(), 2);
        // Total real words: 8
        assert_eq!(doc.total_words, 8);
        // Flat words: 8 real + 1 break = 9
        assert_eq!(doc.words.len(), 9);
    }

    #[test]
    fn whitespace_only_input() {
        let doc = tokenize("   \n\n   ");
        assert_eq!(doc.words.len(), 0);
        assert_eq!(doc.total_words, 0);
    }

    #[test]
    fn sentence_ending_exclamation() {
        let doc = tokenize("Wow! Cool.");
        assert_eq!(doc.paragraphs[0].sentences.len(), 2);
    }

    #[test]
    fn sentence_ending_question() {
        let doc = tokenize("Why? Because.");
        assert_eq!(doc.paragraphs[0].sentences.len(), 2);
    }

    #[test]
    fn no_sentence_ending_punctuation() {
        let doc = tokenize("Hello world no period");
        assert_eq!(doc.paragraphs[0].sentences.len(), 1);
        assert_eq!(doc.total_words, 4);
    }

    #[test]
    fn split_paragraphs_basic() {
        let paras = split_paragraphs("Hello\n\nWorld");
        assert_eq!(paras.len(), 2);
        assert_eq!(paras[0], "Hello");
        assert_eq!(paras[1], "World");
    }

    #[test]
    fn split_paragraphs_three_newlines() {
        let paras = split_paragraphs("Hello\n\n\nWorld");
        assert_eq!(paras.len(), 2);
    }

    #[test]
    fn split_sentences_basic() {
        let sents = split_sentences("Hello world. Goodbye world.");
        assert_eq!(sents.len(), 2);
        assert_eq!(sents[0], "Hello world.");
        assert_eq!(sents[1], "Goodbye world.");
    }

    #[test]
    fn split_sentences_no_final_punctuation() {
        let sents = split_sentences("Hello world. Goodbye world");
        assert_eq!(sents.len(), 2);
        assert_eq!(sents[0], "Hello world.");
        assert_eq!(sents[1], "Goodbye world");
    }
}
