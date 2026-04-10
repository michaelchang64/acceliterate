use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::app::App;

/// Handle a key event, updating app/session state accordingly.
pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Only handle Press events (crossterm sends Press, Release, Repeat on some platforms)
    if key.kind != KeyEventKind::Press {
        return;
    }

    // If help overlay is showing, only handle ? (close help) and q (quit)
    if app.show_help {
        match key.code {
            KeyCode::Char('?') => app.show_help = false,
            KeyCode::Char('q') => app.should_quit = true,
            _ => {} // swallow all other keys
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char(' ') => app.session.toggle(),
        KeyCode::Left => app.session.jump_back_sentence(),
        KeyCode::Right => app.session.jump_forward_sentence(),
        KeyCode::Up => app.session.adjust_wpm(25),
        KeyCode::Down => app.session.adjust_wpm(-25),
        KeyCode::Char('.') => app.zoom_in(),
        KeyCode::Char(',') => app.zoom_out(),
        KeyCode::Char('0') => app.session.restart(),
        KeyCode::Char('b') => app.big_text = !app.big_text,
        KeyCode::Tab => app.toggle_mode(),
        KeyCode::Char('?') => app.show_help = !app.show_help,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    use crate::app::App;
    use crate::core::config::ReaderConfig;
    use crate::core::document::{Document, Word};
    use crate::core::reader::ReadingSession;

    /// Helper: create a minimal Document for testing.
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

    fn make_app() -> App {
        let doc = dummy_document();
        let config = ReaderConfig::default();
        let session = ReadingSession::new(doc, config);
        App::new(session)
    }

    fn press_key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn release_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        }
    }

    // --- Key mapping tests ---

    #[test]
    fn q_sets_should_quit() {
        let mut app = make_app();
        handle_key(&mut app, press_key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn space_toggles_play() {
        let mut app = make_app();
        handle_key(&mut app, press_key(KeyCode::Char(' ')));
        // toggle should have been called — playing state depends on core impl
    }

    #[test]
    fn question_mark_toggles_help() {
        let mut app = make_app();
        assert!(!app.show_help);
        handle_key(&mut app, press_key(KeyCode::Char('?')));
        assert!(app.show_help);
        handle_key(&mut app, press_key(KeyCode::Char('?')));
        assert!(!app.show_help);
    }

    #[test]
    fn arrow_keys_navigate() {
        let mut app = make_app();
        // These call session methods; we just verify they don't panic
        handle_key(&mut app, press_key(KeyCode::Left));
        handle_key(&mut app, press_key(KeyCode::Right));
    }

    #[test]
    fn up_down_adjust_wpm() {
        let mut app = make_app();
        handle_key(&mut app, press_key(KeyCode::Up));
        handle_key(&mut app, press_key(KeyCode::Down));
    }

    // --- Help overlay swallows keys ---

    #[test]
    fn help_overlay_swallows_space() {
        let mut app = make_app();
        app.show_help = true;
        // Space should be swallowed (not toggle play)
        handle_key(&mut app, press_key(KeyCode::Char(' ')));
        assert!(app.show_help); // still showing
    }

    #[test]
    fn help_overlay_allows_quit() {
        let mut app = make_app();
        app.show_help = true;
        handle_key(&mut app, press_key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn help_overlay_closes_with_question_mark() {
        let mut app = make_app();
        app.show_help = true;
        handle_key(&mut app, press_key(KeyCode::Char('?')));
        assert!(!app.show_help);
    }

    // --- Release events are ignored ---

    #[test]
    fn release_events_are_ignored() {
        let mut app = make_app();
        handle_key(&mut app, release_key(KeyCode::Char('q')));
        assert!(!app.should_quit); // should NOT have quit
    }

    // --- Zoom keys ---

    #[test]
    fn period_zooms_in() {
        let mut app = make_app();
        assert_eq!(app.zoom_level, 1);
        handle_key(&mut app, press_key(KeyCode::Char('.')));
        assert_eq!(app.zoom_level, 2);
    }

    #[test]
    fn comma_zooms_out() {
        let mut app = make_app();
        app.zoom_level = 2;
        handle_key(&mut app, press_key(KeyCode::Char(',')));
        assert_eq!(app.zoom_level, 1);
    }

    // --- Mode toggle ---

    #[test]
    fn tab_cycles_modes() {
        let mut app = make_app();
        assert_eq!(app.mode, crate::app::ReadingMode::Rsvp);
        handle_key(&mut app, press_key(KeyCode::Tab));
        assert_eq!(app.mode, crate::app::ReadingMode::Scroll);
        handle_key(&mut app, press_key(KeyCode::Tab));
        assert_eq!(app.mode, crate::app::ReadingMode::Focus);
        handle_key(&mut app, press_key(KeyCode::Tab));
        assert_eq!(app.mode, crate::app::ReadingMode::Rsvp);
    }

    #[test]
    fn zero_restarts() {
        let mut app = make_app();
        app.session.position = 1;
        handle_key(&mut app, press_key(KeyCode::Char('0')));
        assert_eq!(app.session.position, 0);
        assert!(!app.session.is_playing());
    }

    // --- Unknown keys are no-ops ---

    #[test]
    fn unknown_key_is_noop() {
        let mut app = make_app();
        handle_key(&mut app, press_key(KeyCode::Char('x')));
        assert!(!app.should_quit);
        assert!(!app.show_help);
    }
}
