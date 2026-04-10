use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// Render the control hints bar at the bottom.
pub fn render(frame: &mut Frame, area: Rect) {
    let hint_style = Style::default().fg(Color::Gray);

    let line = Line::from(vec![
        Span::styled(
            "Space: Play/Pause | \u{2190}\u{2192}: Sentence | \u{2191}\u{2193}: WPM | Tab: Mode | ,/.: Zoom | b: Big | ?: Help | q: Quit",
            hint_style,
        ),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    #[test]
    fn hint_text_contains_all_keys() {
        let hint = "Space: Play/Pause | \u{2190}\u{2192}: Sentence | \u{2191}\u{2193}: WPM | Tab: Mode | ,/.: Zoom | b: Big | ?: Help | q: Quit";
        assert!(hint.contains("Space"));
        assert!(hint.contains("Play/Pause"));
        assert!(hint.contains("Sentence"));
        assert!(hint.contains("WPM"));
        assert!(hint.contains("Help"));
        assert!(hint.contains("Quit"));
        assert!(hint.contains("?"));
        assert!(hint.contains("q"));
    }
}
