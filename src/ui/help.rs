use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// All keybinding entries displayed in the help overlay.
pub const HELP_BINDINGS: &[(&str, &str)] = &[
    ("Space", "Play / Pause"),
    ("\u{2190}", "Jump back 1 sentence"),
    ("\u{2192}", "Jump forward 1 sentence"),
    ("\u{2191}", "Increase WPM by 25"),
    ("\u{2193}", "Decrease WPM by 25"),
    ("Tab", "Toggle RSVP / Scroll mode"),
    (".", "Zoom in (wider spacing)"),
    (",", "Zoom out (tighter spacing)"),
    ("b", "Toggle big text mode"),
    ("?", "Toggle this help"),
    ("q", "Quit"),
];

/// Render the full-screen help overlay listing all keybindings.
pub fn render(frame: &mut Frame) {
    let full_area = frame.area();

    // Centered overlay: ~60% width, ~70% height.
    let overlay = centered_rect(60, 70, full_area);

    // Clear the background behind the overlay.
    frame.render_widget(Clear, overlay);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title("Help \u{2014} Keybindings")
        .title_alignment(Alignment::Center);

    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    let mut lines: Vec<Line> = Vec::new();

    // Empty line for spacing at top.
    lines.push(Line::from(""));

    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::White);
    let separator_style = Style::default().fg(Color::DarkGray);

    for (key, description) in HELP_BINDINGS {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<8}", key), key_style),
            Span::styled(" \u{2014} ", separator_style),
            Span::styled(*description, desc_style),
        ]));
    }

    // Footer spacing + close instruction.
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press ? to close",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center);

    frame.render_widget(paragraph, inner);
}

/// Calculate a centered rectangle of the given percentage of the parent area.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn help_bindings_contains_all_keys() {
        let keys: Vec<&str> = HELP_BINDINGS.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"Space"));
        assert!(keys.contains(&"\u{2190}")); // left arrow
        assert!(keys.contains(&"\u{2192}")); // right arrow
        assert!(keys.contains(&"\u{2191}")); // up arrow
        assert!(keys.contains(&"\u{2193}")); // down arrow
        assert!(keys.contains(&"?"));
        assert!(keys.contains(&"q"));
    }

    #[test]
    fn help_bindings_has_eleven_entries() {
        assert_eq!(HELP_BINDINGS.len(), 11);
    }

    #[test]
    fn help_descriptions_not_empty() {
        for (key, desc) in HELP_BINDINGS {
            assert!(!key.is_empty(), "Key should not be empty");
            assert!(!desc.is_empty(), "Description for '{}' should not be empty", key);
        }
    }

    #[test]
    fn centered_rect_produces_valid_area() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(60, 70, area);

        // The centered rect should be smaller than the parent.
        assert!(centered.width <= area.width);
        assert!(centered.height <= area.height);

        // Should be roughly 60% of 100 = 60 wide.
        assert!(centered.width >= 55 && centered.width <= 65,
            "Expected width ~60, got {}", centered.width);

        // Should be roughly 70% of 50 = 35 tall.
        assert!(centered.height >= 30 && centered.height <= 40,
            "Expected height ~35, got {}", centered.height);

        // Should be within the parent bounds.
        assert!(centered.x + centered.width <= area.x + area.width);
        assert!(centered.y + centered.height <= area.y + area.height);
    }

    #[test]
    fn centered_rect_is_actually_centered() {
        let area = Rect::new(0, 0, 100, 50);
        let centered = centered_rect(60, 70, area);

        // The horizontal center of the popup should be close to the parent center.
        let parent_center_x = area.x + area.width / 2;
        let popup_center_x = centered.x + centered.width / 2;
        let diff_x = (parent_center_x as i32 - popup_center_x as i32).unsigned_abs();
        assert!(diff_x <= 2, "Horizontal centering off by {}", diff_x);

        let parent_center_y = area.y + area.height / 2;
        let popup_center_y = centered.y + centered.height / 2;
        let diff_y = (parent_center_y as i32 - popup_center_y as i32).unsigned_abs();
        assert!(diff_y <= 2, "Vertical centering off by {}", diff_y);
    }

    #[test]
    fn centered_rect_with_small_area() {
        let area = Rect::new(0, 0, 20, 10);
        let centered = centered_rect(60, 70, area);

        // Should still be within bounds even for small terminals.
        assert!(centered.x + centered.width <= area.x + area.width);
        assert!(centered.y + centered.height <= area.y + area.height);
    }
}
