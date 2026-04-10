use crate::core::reader::ReadingSession;

/// Active reading mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingMode {
    Rsvp,
    Scroll,
}

/// TUI application state. Wraps ReadingSession + UI-specific state.
pub struct App {
    pub session: ReadingSession,
    pub show_help: bool,
    pub should_quit: bool,
    /// Zoom level for RSVP display (1=normal, 2=spaced, 3=extra-spaced).
    pub zoom_level: u8,
    /// When true, render words using 5×5 block font.
    pub big_text: bool,
    pub mode: ReadingMode,
}

impl App {
    pub fn new(session: ReadingSession) -> Self {
        Self {
            session,
            show_help: false,
            should_quit: false,
            zoom_level: 1,
            big_text: false,
            mode: ReadingMode::Rsvp,
        }
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ReadingMode::Rsvp => ReadingMode::Scroll,
            ReadingMode::Scroll => ReadingMode::Rsvp,
        };
    }

    pub fn zoom_in(&mut self) {
        if self.zoom_level < 3 {
            self.zoom_level += 1;
        }
    }

    pub fn zoom_out(&mut self) {
        if self.zoom_level > 1 {
            self.zoom_level -= 1;
        }
    }

    /// Returns true when the reader has reached the end of the document.
    #[allow(dead_code)]
    pub fn is_finished(&self) -> bool {
        self.session.position >= self.session.document.words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::ReaderConfig;
    use crate::core::document::{Document, Word};

    /// Helper: create a minimal Document with one word for testing.
    fn dummy_document() -> Document {
        let word = Word {
            text: "hello".to_string(),
            orp_index: 1,
            base_delay_multiplier: 1.0,
            punctuation_delay_multiplier: 0.0,
            is_paragraph_break: false,
        };
        Document {
            paragraphs: vec![],
            words: vec![word],
            word_positions: vec![(0, 0, 0)],
            total_words: 1,
        }
    }

    #[test]
    fn new_app_defaults() {
        let doc = dummy_document();
        let config = ReaderConfig::default();
        let session = ReadingSession::new(doc, config);
        let app = App::new(session);

        assert!(!app.show_help);
        assert!(!app.should_quit);
        assert_eq!(app.zoom_level, 1);
        assert_eq!(app.mode, ReadingMode::Rsvp);
    }

    #[test]
    fn toggle_mode() {
        let doc = dummy_document();
        let config = ReaderConfig::default();
        let session = ReadingSession::new(doc, config);
        let mut app = App::new(session);

        assert_eq!(app.mode, ReadingMode::Rsvp);
        app.toggle_mode();
        assert_eq!(app.mode, ReadingMode::Scroll);
        app.toggle_mode();
        assert_eq!(app.mode, ReadingMode::Rsvp);
    }

    #[test]
    fn zoom_in_and_out() {
        let doc = dummy_document();
        let config = ReaderConfig::default();
        let session = ReadingSession::new(doc, config);
        let mut app = App::new(session);

        assert_eq!(app.zoom_level, 1);
        app.zoom_in();
        assert_eq!(app.zoom_level, 2);
        app.zoom_in();
        assert_eq!(app.zoom_level, 3);
        app.zoom_in(); // clamps at 3
        assert_eq!(app.zoom_level, 3);
        app.zoom_out();
        assert_eq!(app.zoom_level, 2);
        app.zoom_out();
        assert_eq!(app.zoom_level, 1);
        app.zoom_out(); // clamps at 1
        assert_eq!(app.zoom_level, 1);
    }

    #[test]
    fn is_finished_false_at_start() {
        let doc = dummy_document();
        let config = ReaderConfig::default();
        let session = ReadingSession::new(doc, config);
        let app = App::new(session);

        assert!(!app.is_finished());
    }

    #[test]
    fn is_finished_true_at_end() {
        let doc = dummy_document();
        let config = ReaderConfig::default();
        let mut session = ReadingSession::new(doc, config);
        // Move position past the end
        session.position = session.document.words.len();
        let app = App::new(session);

        assert!(app.is_finished());
    }
}
