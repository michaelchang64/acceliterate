use std::time::Duration;

/// Compute the display duration for a word given the target WPM.
/// Accounts for word length and trailing punctuation per the variable timing table.
///
/// Final duration = base_delay * base_delay_multiplier + base_delay * punctuation_delay_multiplier
pub fn word_duration(wpm: u32, base_delay_multiplier: f32, punctuation_delay_multiplier: f32) -> Duration {
    let base_delay_ms = 60_000.0 / wpm as f32;
    let total_ms = base_delay_ms * base_delay_multiplier + base_delay_ms * punctuation_delay_multiplier;
    Duration::from_millis(total_ms.round() as u64)
}

/// Compute the base_delay_multiplier from word length.
///
/// - 1-3 chars: 0.85x
/// - 4-7 chars: 1.0x
/// - 8-11 chars: 1.3x
/// - 12+ chars: 1.6x
pub fn length_multiplier(word_len: usize) -> f32 {
    match word_len {
        0..=3 => 0.85,
        4..=7 => 1.0,
        8..=11 => 1.3,
        _ => 1.6,
    }
}

/// Compute the punctuation_delay_multiplier from trailing punctuation.
///
/// - Comma, semicolon, colon: +0.4x
/// - Period, !, ?: +1.0x
/// - No trailing punctuation: +0.0x
pub fn punctuation_multiplier(word: &str) -> f32 {
    match word.chars().last() {
        Some(',' | ';' | ':') => 0.4,
        Some('.' | '!' | '?') => 1.0,
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- length_multiplier tests ---

    #[test]
    fn length_zero() {
        assert_eq!(length_multiplier(0), 0.85);
    }

    #[test]
    fn length_short_1() {
        assert_eq!(length_multiplier(1), 0.85);
    }

    #[test]
    fn length_short_3() {
        assert_eq!(length_multiplier(3), 0.85);
    }

    #[test]
    fn length_medium_4() {
        assert_eq!(length_multiplier(4), 1.0);
    }

    #[test]
    fn length_medium_7() {
        assert_eq!(length_multiplier(7), 1.0);
    }

    #[test]
    fn length_long_8() {
        assert_eq!(length_multiplier(8), 1.3);
    }

    #[test]
    fn length_long_11() {
        assert_eq!(length_multiplier(11), 1.3);
    }

    #[test]
    fn length_very_long_12() {
        assert_eq!(length_multiplier(12), 1.6);
    }

    #[test]
    fn length_very_long_20() {
        assert_eq!(length_multiplier(20), 1.6);
    }

    // --- punctuation_multiplier tests ---

    #[test]
    fn no_punctuation() {
        assert_eq!(punctuation_multiplier("hello"), 0.0);
    }

    #[test]
    fn trailing_comma() {
        assert_eq!(punctuation_multiplier("hello,"), 0.4);
    }

    #[test]
    fn trailing_semicolon() {
        assert_eq!(punctuation_multiplier("thus;"), 0.4);
    }

    #[test]
    fn trailing_colon() {
        assert_eq!(punctuation_multiplier("note:"), 0.4);
    }

    #[test]
    fn trailing_period() {
        assert_eq!(punctuation_multiplier("end."), 1.0);
    }

    #[test]
    fn trailing_exclamation() {
        assert_eq!(punctuation_multiplier("wow!"), 1.0);
    }

    #[test]
    fn trailing_question() {
        assert_eq!(punctuation_multiplier("why?"), 1.0);
    }

    #[test]
    fn empty_word() {
        assert_eq!(punctuation_multiplier(""), 0.0);
    }

    // --- word_duration tests ---

    #[test]
    fn duration_at_300_wpm_normal_word() {
        // base_delay = 60000 / 300 = 200ms
        // 200 * 1.0 + 200 * 0.0 = 200ms
        let d = word_duration(300, 1.0, 0.0);
        assert_eq!(d, Duration::from_millis(200));
    }

    #[test]
    fn duration_at_300_wpm_short_word() {
        // base_delay = 200ms
        // 200 * 0.85 + 200 * 0.0 = 170ms
        let d = word_duration(300, 0.85, 0.0);
        assert_eq!(d, Duration::from_millis(170));
    }

    #[test]
    fn duration_at_300_wpm_with_period() {
        // base_delay = 200ms
        // 200 * 1.0 + 200 * 1.0 = 400ms
        let d = word_duration(300, 1.0, 1.0);
        assert_eq!(d, Duration::from_millis(400));
    }

    #[test]
    fn duration_at_300_wpm_long_word_with_comma() {
        // base_delay = 200ms
        // 200 * 1.3 + 200 * 0.4 = 260 + 80 = 340ms
        let d = word_duration(300, 1.3, 0.4);
        assert_eq!(d, Duration::from_millis(340));
    }

    #[test]
    fn duration_at_60_wpm() {
        // base_delay = 60000 / 60 = 1000ms
        // 1000 * 1.0 + 1000 * 0.0 = 1000ms
        let d = word_duration(60, 1.0, 0.0);
        assert_eq!(d, Duration::from_millis(1000));
    }
}
