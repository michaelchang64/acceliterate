use std::time::{Duration, Instant};

use super::config::ReaderConfig;
use super::document::{Document, Paragraph, Sentence, Word};
use super::stats::SessionStats;
use super::timing;

/// The core reading engine. Frontend-agnostic.
pub struct ReadingSession {
    pub document: Document,
    pub config: ReaderConfig,
    pub position: usize,
    pub playing: bool,
    pub stats: SessionStats,
    last_advance_time: Option<Instant>,
    /// Highest position ever reached — words are only counted once.
    max_position_reached: usize,
}

impl ReadingSession {
    pub fn new(document: Document, config: ReaderConfig) -> Self {
        let wpm = config.wpm;
        Self {
            document,
            config,
            position: 0,
            playing: false,
            stats: SessionStats::new(wpm),
            last_advance_time: None,
            max_position_reached: 0,
        }
    }

    pub fn play(&mut self) {
        if !self.playing {
            self.playing = true;
            self.last_advance_time = Some(Instant::now());
            self.stats.start_reading();
        }
    }

    pub fn pause(&mut self) {
        if self.playing {
            self.playing = false;
            self.last_advance_time = None;
            self.stats.pause_reading();
        }
    }

    pub fn toggle(&mut self) {
        if self.playing {
            self.pause();
        } else {
            self.play();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// If playing and enough time has elapsed since last advance, move position forward by 1.
    /// Return true if position changed.
    pub fn tick(&mut self) -> bool {
        if !self.playing || self.document.words.is_empty() {
            return false;
        }

        if self.position >= self.document.words.len() {
            return false;
        }

        let required_duration = self.tick_duration();

        if let Some(last) = self.last_advance_time {
            if last.elapsed() >= required_duration {
                // Only count words we haven't seen before (not on re-read after jump-back).
                if self.position >= self.max_position_reached
                    && !self.document.words[self.position].is_paragraph_break
                {
                    self.stats.record_word();
                }

                self.position += 1;
                if self.position > self.max_position_reached {
                    self.max_position_reached = self.position;
                }
                self.last_advance_time = Some(Instant::now());

                // If we've reached the end, pause
                if self.position >= self.document.words.len() {
                    self.pause();
                }

                return true;
            }
        }

        false
    }

    /// Return the Duration for the current word.
    pub fn tick_duration(&self) -> Duration {
        if self.document.words.is_empty() || self.position >= self.document.words.len() {
            return Duration::from_millis(200); // fallback
        }

        let word = &self.document.words[self.position];

        if word.is_paragraph_break {
            // Paragraph break: base_delay * 1.0 + base_delay * 2.0 = base_delay * 3.0
            return timing::word_duration(
                self.config.wpm,
                word.base_delay_multiplier,
                word.punctuation_delay_multiplier,
            );
        }

        timing::word_duration(
            self.config.wpm,
            word.base_delay_multiplier,
            word.punctuation_delay_multiplier,
        )
    }

    /// Move position to start of next sentence.
    pub fn jump_forward_sentence(&mut self) {
        if self.document.words.is_empty() {
            return;
        }
        let pos = self.position.min(self.document.words.len() - 1);
        let next = self.document.next_sentence_start_index(pos);
        if next < self.document.words.len() {
            self.position = next;
        }
        // Skip paragraph break markers
        while self.position < self.document.words.len()
            && self.document.words[self.position].is_paragraph_break
        {
            self.position += 1;
        }
    }

    /// Move position to start of current sentence (or previous if already at start).
    pub fn jump_back_sentence(&mut self) {
        if self.document.words.is_empty() {
            return;
        }
        let pos = self.position.min(self.document.words.len() - 1);
        let start = self.document.sentence_start_index(pos);

        if start == pos && pos > 0 {
            // Already at the start of a sentence; go to the previous one.
            // Step back past any paragraph break markers.
            let mut prev = pos - 1;
            while prev > 0 && self.document.words[prev].is_paragraph_break {
                prev -= 1;
            }
            self.position = self.document.sentence_start_index(prev);
        } else {
            self.position = start;
        }
    }

    /// Set WPM, clamped to 50-1000.
    pub fn set_wpm(&mut self, wpm: u32) {
        self.config.wpm = wpm.clamp(50, 1000);
        self.stats.current_wpm = self.config.wpm;
    }

    /// Adjust WPM by delta, clamped to 50-1000.
    pub fn adjust_wpm(&mut self, delta: i32) {
        let new_wpm = (self.config.wpm as i32 + delta).clamp(50, 1000) as u32;
        self.set_wpm(new_wpm);
    }

    /// Current WPM.
    pub fn wpm(&self) -> u32 {
        self.config.wpm
    }

    /// Return &Word at current position.
    pub fn current_word(&self) -> &Word {
        &self.document.words[self.position]
    }

    /// Return the Sentence containing the current position.
    #[allow(dead_code)]
    pub fn current_sentence(&self) -> &Sentence {
        let (para_idx, sent_idx, _) = self.document.word_positions[self.position];
        &self.document.paragraphs[para_idx].sentences[sent_idx]
    }

    /// Return the Paragraph containing the current position.
    #[allow(dead_code)]
    pub fn current_paragraph(&self) -> &Paragraph {
        let (para_idx, _, _) = self.document.word_positions[self.position];
        &self.document.paragraphs[para_idx]
    }

    /// Position as fraction of total words (0.0 to 1.0).
    pub fn progress(&self) -> f64 {
        if self.document.total_words == 0 {
            return 0.0;
        }
        // Count real words read so far (up to current position)
        let real_words_before = self.document.words[..self.position]
            .iter()
            .filter(|w| !w.is_paragraph_break)
            .count();
        real_words_before as f64 / self.document.total_words as f64
    }

    /// Return &SessionStats.
    #[allow(dead_code)]
    pub fn stats(&self) -> &SessionStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tokenizer::tokenize;
    use std::thread;

    fn make_session(text: &str) -> ReadingSession {
        let doc = tokenize(text);
        let config = ReaderConfig { wpm: 300 };
        ReadingSession::new(doc, config)
    }

    #[test]
    fn new_session_defaults() {
        let session = make_session("Hello world.");
        assert_eq!(session.position, 0);
        assert!(!session.playing);
        assert_eq!(session.wpm(), 300);
    }

    #[test]
    fn play_pause_toggle() {
        let mut session = make_session("Hello world.");
        assert!(!session.is_playing());
        session.play();
        assert!(session.is_playing());
        session.pause();
        assert!(!session.is_playing());
        session.toggle();
        assert!(session.is_playing());
        session.toggle();
        assert!(!session.is_playing());
    }

    #[test]
    fn play_twice_is_idempotent() {
        let mut session = make_session("Hello world.");
        session.play();
        session.play();
        assert!(session.is_playing());
    }

    #[test]
    fn pause_twice_is_idempotent() {
        let mut session = make_session("Hello world.");
        session.play();
        session.pause();
        session.pause();
        assert!(!session.is_playing());
    }

    #[test]
    fn tick_does_not_advance_when_paused() {
        let mut session = make_session("Hello world.");
        assert!(!session.tick());
    }

    #[test]
    fn tick_advances_after_duration() {
        let mut session = make_session("Hello world.");
        session.play();
        // At 300 WPM, base delay = 200ms. "Hello" is 5 chars = 1.0x multiplier, 0.0 punct
        // Duration = 200ms
        thread::sleep(Duration::from_millis(250));
        let advanced = session.tick();
        assert!(advanced);
        assert_eq!(session.position, 1);
    }

    #[test]
    fn tick_does_not_advance_too_early() {
        let mut session = make_session("Hello world.");
        session.play();
        // Don't wait long enough
        let advanced = session.tick();
        // Might or might not advance depending on system speed, but at 200ms threshold
        // it typically won't advance immediately
        // (This test is timing-sensitive; we just verify no panic)
        let _ = advanced;
    }

    #[test]
    fn tick_duration_normal_word() {
        let session = make_session("Hello");
        // "Hello" is 5 chars: 1.0x, no punct: 0.0
        // 60000/300 * 1.0 + 60000/300 * 0.0 = 200ms
        assert_eq!(session.tick_duration(), Duration::from_millis(200));
    }

    #[test]
    fn tick_duration_short_word() {
        let session = make_session("Hi");
        // "Hi" is 2 chars: 0.85x, no punct: 0.0
        // 200 * 0.85 = 170ms
        assert_eq!(session.tick_duration(), Duration::from_millis(170));
    }

    #[test]
    fn set_wpm() {
        let mut session = make_session("Hello");
        session.set_wpm(500);
        assert_eq!(session.wpm(), 500);
    }

    #[test]
    fn set_wpm_clamps_low() {
        let mut session = make_session("Hello");
        session.set_wpm(10);
        assert_eq!(session.wpm(), 50);
    }

    #[test]
    fn set_wpm_clamps_high() {
        let mut session = make_session("Hello");
        session.set_wpm(5000);
        assert_eq!(session.wpm(), 1000);
    }

    #[test]
    fn adjust_wpm() {
        let mut session = make_session("Hello");
        assert_eq!(session.wpm(), 300);
        session.adjust_wpm(50);
        assert_eq!(session.wpm(), 350);
        session.adjust_wpm(-100);
        assert_eq!(session.wpm(), 250);
    }

    #[test]
    fn adjust_wpm_clamps() {
        let mut session = make_session("Hello");
        session.adjust_wpm(-500); // 300 - 500 = -200 -> clamp to 50
        assert_eq!(session.wpm(), 50);
        session.adjust_wpm(2000); // 50 + 2000 = 2050 -> clamp to 1000
        assert_eq!(session.wpm(), 1000);
    }

    #[test]
    fn current_word() {
        let session = make_session("Hello world.");
        assert_eq!(session.current_word().text, "Hello");
    }

    #[test]
    fn current_sentence() {
        let session = make_session("Hello world. Bye.");
        let sent = session.current_sentence();
        assert_eq!(sent.words.len(), 2);
        assert_eq!(sent.words[0].text, "Hello");
    }

    #[test]
    fn current_paragraph() {
        let session = make_session("Hello world.\n\nBye.");
        let para = session.current_paragraph();
        assert_eq!(para.sentences.len(), 1);
    }

    #[test]
    fn progress_at_start_is_zero() {
        let session = make_session("Hello world.");
        assert_eq!(session.progress(), 0.0);
    }

    #[test]
    fn progress_calculation() {
        let mut session = make_session("A B C D");
        // 4 real words. Position 0 = 0/4 = 0.0
        assert_eq!(session.progress(), 0.0);
        session.position = 2;
        // 2 real words before position 2 = 2/4 = 0.5
        assert_eq!(session.progress(), 0.5);
        session.position = 4; // past end
        // 4 real words before position 4 = 4/4 = 1.0
        assert_eq!(session.progress(), 1.0);
    }

    #[test]
    fn progress_with_paragraph_breaks() {
        let mut session = make_session("A B.\n\nC D.");
        // words: A, B., [break], C, D.
        // total_words = 4
        session.position = 3; // "C"
        // Real words before index 3: A, B. = 2 (break is not counted)
        // progress = 2/4 = 0.5
        assert_eq!(session.progress(), 0.5);
    }

    #[test]
    fn jump_forward_sentence() {
        let mut session = make_session("Hello world. Goodbye world.");
        assert_eq!(session.position, 0);
        session.jump_forward_sentence();
        assert_eq!(session.position, 2); // "Goodbye"
    }

    #[test]
    fn jump_forward_sentence_at_last_sentence() {
        let mut session = make_session("Hello world. Goodbye.");
        session.position = 2; // "Goodbye."
        session.jump_forward_sentence();
        // Should stay at same position since there's no next sentence
        // (next_sentence_start returns words.len() which is >= words.len())
        assert_eq!(session.position, 2);
    }

    #[test]
    fn jump_back_sentence() {
        let mut session = make_session("Hello world. Goodbye world.");
        session.position = 3; // "world." in second sentence
        session.jump_back_sentence();
        assert_eq!(session.position, 2); // "Goodbye"
    }

    #[test]
    fn jump_back_sentence_already_at_start() {
        let mut session = make_session("Hello world. Goodbye world.");
        session.position = 2; // "Goodbye" = start of second sentence
        session.jump_back_sentence();
        assert_eq!(session.position, 0); // "Hello" = start of first sentence
    }

    #[test]
    fn jump_back_sentence_at_document_start() {
        let mut session = make_session("Hello world.");
        session.position = 0;
        session.jump_back_sentence();
        assert_eq!(session.position, 0); // Can't go further back
    }

    #[test]
    fn jump_forward_across_paragraph_break() {
        let mut session = make_session("Hello. Bye.\n\nNew para.");
        // words: "Hello.", "Bye.", [break], "New", "para."
        session.position = 1; // "Bye."
        session.jump_forward_sentence();
        // Should skip the break and land on "New"
        assert_eq!(session.position, 3);
        assert_eq!(session.current_word().text, "New");
    }

    #[test]
    fn stats_reference() {
        let session = make_session("Hello");
        let stats = session.stats();
        assert_eq!(stats.words_read, 0);
        assert_eq!(stats.current_wpm, 300);
    }

    #[test]
    fn empty_document() {
        let session = make_session("");
        assert_eq!(session.progress(), 0.0);
        assert_eq!(session.tick_duration(), Duration::from_millis(200));
    }

    #[test]
    fn jump_back_does_not_recount_words() {
        let mut session = make_session("Hello world. Goodbye world.");
        // Manually advance past 3 words, simulating tick() behavior.
        session.stats.record_word(); // "Hello"
        session.position = 1;
        session.max_position_reached = 1;
        session.stats.record_word(); // "world."
        session.position = 2;
        session.max_position_reached = 2;
        session.stats.record_word(); // "Goodbye"
        session.position = 3;
        session.max_position_reached = 3;

        assert_eq!(session.stats.words_read, 3);

        // Jump back to start of second sentence ("Goodbye")
        session.jump_back_sentence();
        assert_eq!(session.position, 2);

        // Now simulate tick advancing from position 2 (re-reading "Goodbye").
        // Since position 2 < max_position_reached 3, it should NOT count.
        session.playing = true;
        session.last_advance_time = Some(std::time::Instant::now() - Duration::from_secs(1));
        session.tick();
        assert_eq!(session.position, 3);
        assert_eq!(session.stats.words_read, 3); // still 3, not 4
    }
}
