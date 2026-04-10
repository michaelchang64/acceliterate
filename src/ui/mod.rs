pub mod bigfont;
pub mod controls;
pub mod help;
pub mod rsvp;
pub mod scroll;
pub mod stats;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::{App, ReadingMode};

/// Top-level render function. Delegates to sub-renderers.
pub fn render(app: &App, frame: &mut Frame) {
    if app.show_help {
        render_main_layout(app, frame);
        help::render(frame);
    } else {
        render_main_layout(app, frame);
    }
}

/// Render the three-panel main layout: Stats | Reader | Controls.
fn render_main_layout(app: &App, frame: &mut Frame) {
    let chunks = build_layout(frame.area());

    stats::render(app, frame, chunks[0]);
    match app.mode {
        ReadingMode::Rsvp => rsvp::render(app, frame, chunks[1]),
        ReadingMode::Scroll => scroll::render(app, frame, chunks[1]),
    }
    controls::render(frame, chunks[2]);
}

/// Split a given area into the three vertical regions.
/// Returns a Vec of 3 Rects: [stats, rsvp, controls].
pub fn build_layout(area: ratatui::layout::Rect) -> std::rc::Rc<[ratatui::layout::Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Stats panel
            Constraint::Min(5),   // RSVP display (fill remaining)
            Constraint::Length(1), // Control hints bar
        ])
        .split(area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn layout_splits_correctly_for_standard_terminal() {
        // Simulate a typical 80x24 terminal
        let area = Rect::new(0, 0, 80, 24);
        let chunks = build_layout(area);

        assert_eq!(chunks.len(), 3);

        // Stats panel: 3 lines
        assert_eq!(chunks[0].height, 3);
        assert_eq!(chunks[0].y, 0);
        assert_eq!(chunks[0].width, 80);

        // RSVP area: remaining space = 24 - 3 - 1 = 20
        assert_eq!(chunks[1].height, 20);
        assert_eq!(chunks[1].y, 3);

        // Controls bar: 1 line
        assert_eq!(chunks[2].height, 1);
        assert_eq!(chunks[2].y, 23);
    }

    #[test]
    fn layout_splits_correctly_for_small_terminal() {
        // Minimal terminal: 40x10
        let area = Rect::new(0, 0, 40, 10);
        let chunks = build_layout(area);

        assert_eq!(chunks[0].height, 3);
        // RSVP gets 10 - 3 - 1 = 6 (>= Min(5))
        assert_eq!(chunks[1].height, 6);
        assert_eq!(chunks[2].height, 1);
    }

    #[test]
    fn layout_splits_correctly_for_large_terminal() {
        let area = Rect::new(0, 0, 200, 60);
        let chunks = build_layout(area);

        assert_eq!(chunks[0].height, 3);
        // RSVP gets 60 - 3 - 1 = 56
        assert_eq!(chunks[1].height, 56);
        assert_eq!(chunks[2].height, 1);
    }

    #[test]
    fn layout_widths_match_parent() {
        let area = Rect::new(5, 10, 100, 50);
        let chunks = build_layout(area);

        for chunk in chunks.iter() {
            assert_eq!(chunk.width, 100);
            assert_eq!(chunk.x, 5);
        }
    }
}
