/// Compute the ORP (Optimal Recognition Point) index for a word.
/// ORP is at ~25% into the word, capped at index 4.
/// Based on O'Regan & Jacobs (1992).
pub fn orp_index(word_len: usize) -> usize {
    if word_len <= 1 {
        return 0;
    }
    let idx = word_len as f64 * 0.25;
    (idx.floor() as usize).min(4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_word() {
        assert_eq!(orp_index(0), 0);
    }

    #[test]
    fn single_char() {
        assert_eq!(orp_index(1), 0);
    }

    #[test]
    fn two_chars() {
        // floor(2 * 0.25) = floor(0.5) = 0
        assert_eq!(orp_index(2), 0);
    }

    #[test]
    fn three_chars() {
        // floor(3 * 0.25) = floor(0.75) = 0
        assert_eq!(orp_index(3), 0);
    }

    #[test]
    fn four_chars() {
        // floor(4 * 0.25) = 1
        assert_eq!(orp_index(4), 1);
    }

    #[test]
    fn five_chars() {
        // floor(5 * 0.25) = floor(1.25) = 1
        assert_eq!(orp_index(5), 1);
    }

    #[test]
    fn eight_chars() {
        // floor(8 * 0.25) = 2
        assert_eq!(orp_index(8), 2);
    }

    #[test]
    fn twelve_chars() {
        // floor(12 * 0.25) = 3
        assert_eq!(orp_index(12), 3);
    }

    #[test]
    fn sixteen_chars() {
        // floor(16 * 0.25) = 4
        assert_eq!(orp_index(16), 4);
    }

    #[test]
    fn twenty_chars_capped_at_4() {
        // floor(20 * 0.25) = 5, capped at 4
        assert_eq!(orp_index(20), 4);
    }

    #[test]
    fn very_long_word_capped_at_4() {
        assert_eq!(orp_index(100), 4);
    }
}
