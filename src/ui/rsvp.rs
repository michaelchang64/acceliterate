use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render the RSVP word display with ORP highlighting and redicle markers.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let playing = app.session.is_playing();
    let status_icon = if playing { " \u{25b6}" } else { " \u{23f8}" };
    let big_label = if app.big_text { " [BIG·Latin only]" } else { "" };
    let title = format!("RSVP{}{}", status_icon, big_label);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Check if we're past the end of the document.
    if app.session.position >= app.session.document.words.len() {
        let end_msg = Paragraph::new(Line::from(Span::styled(
            "End of document",
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        let y_offset = inner.height / 2;
        if y_offset < inner.height {
            let msg_area = Rect::new(inner.x, inner.y + y_offset, inner.width, 1);
            frame.render_widget(end_msg, msg_area);
        }
        return;
    }

    // Show "PRESS SPACE TO START" prompt when paused at the beginning.
    if !app.session.is_playing() && app.session.position == 0 && app.session.stats.words_read == 0 {
        let prompt = Paragraph::new(Line::from(Span::styled(
            "[ PRESS SPACE TO START ]",
            Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        let y_offset = inner.height / 2;
        if y_offset < inner.height {
            let msg_area = Rect::new(inner.x, inner.y + y_offset, inner.width, 1);
            frame.render_widget(prompt, msg_area);
        }
        return;
    }

    let word = app.session.current_word();

    // Paragraph break: show blank area.
    if word.is_paragraph_break {
        return;
    }

    let text = &word.text;
    let orp = word.orp_index;

    if text.is_empty() {
        return;
    }

    // Calculate the center column of the inner area.
    let center_x = inner.width as usize / 2;

    // Redicle marker style.
    let redicle_style = Style::default().fg(Color::DarkGray);

    if app.big_text {
        // Big text mode: 5-row tall block font + 2 redicle rows = 7 rows.
        let offset = super::bigfont::compute_big_text_offset(center_x, orp);
        let word_lines = super::bigfont::render_big_word(text, orp, offset);

        let group_height = (super::bigfont::CHAR_HEIGHT as u16) + 2;
        let vert_start = if inner.height >= group_height {
            inner.y + (inner.height - group_height) / 2
        } else {
            inner.y
        };

        // Top redicle.
        if vert_start < inner.y + inner.height {
            let marker_line = build_marker_line("\u{25bc}", center_x, redicle_style);
            let marker_area = Rect::new(inner.x, vert_start, inner.width, 1);
            frame.render_widget(Paragraph::new(marker_line), marker_area);
        }

        // Word rows.
        for (i, line) in word_lines.into_iter().enumerate() {
            let y = vert_start + 1 + i as u16;
            if y < inner.y + inner.height {
                let row_area = Rect::new(inner.x, y, inner.width, 1);
                frame.render_widget(Paragraph::new(line), row_area);
            }
        }

        // Bottom redicle.
        let bottom_y = vert_start + 1 + super::bigfont::CHAR_HEIGHT as u16;
        if bottom_y < inner.y + inner.height {
            let marker_line = build_marker_line("\u{25b2}", center_x, redicle_style);
            let marker_area = Rect::new(inner.x, bottom_y, inner.width, 1);
            frame.render_widget(Paragraph::new(marker_line), marker_area);
        }
    } else {
        // Normal mode: single word line + 2 redicle rows = 3 rows.
        let zoom = app.zoom_level;
        let offset = compute_word_offset(center_x, orp, zoom);

        let group_height = 3u16;
        let vert_start = if inner.height >= group_height {
            inner.y + (inner.height - group_height) / 2
        } else {
            inner.y
        };

        // Top redicle.
        if vert_start >= inner.y && vert_start < inner.y + inner.height {
            let marker_line = build_marker_line("\u{25bc}", center_x, redicle_style);
            let marker_area = Rect::new(inner.x, vert_start, inner.width, 1);
            frame.render_widget(Paragraph::new(marker_line), marker_area);
        }

        // Word line.
        let word_y = vert_start + 1;
        if word_y >= inner.y && word_y < inner.y + inner.height {
            let padded_line = build_padded_word_line(text, orp, offset, zoom);
            let word_area = Rect::new(inner.x, word_y, inner.width, 1);
            frame.render_widget(Paragraph::new(padded_line), word_area);
        }

        // Bottom redicle.
        let bottom_y = vert_start + 2;
        if bottom_y >= inner.y && bottom_y < inner.y + inner.height {
            let marker_line = build_marker_line("\u{25b2}", center_x, redicle_style);
            let marker_area = Rect::new(inner.x, bottom_y, inner.width, 1);
            frame.render_widget(Paragraph::new(marker_line), marker_area);
        }
    }
}

/// Compute the horizontal character offset where the word should start rendering,
/// such that `orp_index` aligns to `center_x`. `zoom` controls letter spacing
/// (1=normal, 2=one space between chars, 3=two spaces between chars).
///
/// Returns the number of leading spaces to prepend.
pub fn compute_word_offset(center_x: usize, orp_index: usize, zoom: u8) -> usize {
    center_x.saturating_sub(orp_index * zoom as usize)
}

/// Build a `Line` with three spans: pre-ORP (default), ORP char (red+bold), post-ORP (default).
#[allow(dead_code)]
pub fn build_orp_line<'a>(text: &'a str, orp_index: usize) -> Line<'a> {
    let orp_style = Style::default()
        .fg(Color::LightGreen)
        .add_modifier(Modifier::BOLD);

    let chars: Vec<char> = text.chars().collect();
    let clamped_orp = orp_index.min(chars.len().saturating_sub(1));

    let pre: String = chars[..clamped_orp].iter().collect();
    let orp_char: String = chars[clamped_orp..=clamped_orp].iter().collect();
    let post: String = chars[clamped_orp + 1..].iter().collect();

    Line::from(vec![
        Span::raw(pre),
        Span::styled(orp_char, orp_style),
        Span::raw(post),
    ])
}

/// Build a redicle marker line with the marker character at the given column position.
fn build_marker_line<'a>(marker: &'a str, col: usize, style: Style) -> Line<'a> {
    let padding = " ".repeat(col);
    Line::from(vec![
        Span::raw(padding),
        Span::styled(marker, style),
    ])
}

/// Build the padded word line: leading spaces + ORP-styled word with letter spacing.
fn build_padded_word_line<'a>(text: &str, orp_index: usize, offset: usize, zoom: u8) -> Line<'a> {
    let word_style = Style::default().add_modifier(Modifier::BOLD);
    let orp_style = Style::default()
        .fg(Color::LightGreen)
        .add_modifier(Modifier::BOLD);

    let chars: Vec<char> = text.chars().collect();
    let clamped_orp = orp_index.min(chars.len().saturating_sub(1));

    let padding = " ".repeat(offset);
    let spacing = " ".repeat((zoom as usize).saturating_sub(1));

    // Build pre-ORP with inter-character spacing, plus trailing spacing before ORP char.
    let pre_chars: Vec<String> = chars[..clamped_orp].iter().map(|c| c.to_string()).collect();
    let pre = if !pre_chars.is_empty() {
        format!("{}{}", pre_chars.join(&spacing), &spacing)
    } else {
        String::new()
    };

    let orp_char: String = chars[clamped_orp..=clamped_orp].iter().collect();

    // Build post-ORP with leading spacing, then inter-character spacing.
    let post_chars: Vec<String> = chars[clamped_orp + 1..].iter().map(|c| c.to_string()).collect();
    let post = if !post_chars.is_empty() {
        format!("{}{}", &spacing, post_chars.join(&spacing))
    } else {
        String::new()
    };

    Line::from(vec![
        Span::raw(padding),
        Span::styled(pre, word_style),
        Span::styled(orp_char, orp_style),
        Span::styled(post, word_style),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_offset_orp_at_start() {
        assert_eq!(compute_word_offset(40, 0, 1), 40);
    }

    #[test]
    fn compute_offset_orp_in_middle() {
        assert_eq!(compute_word_offset(40, 2, 1), 38);
    }

    #[test]
    fn compute_offset_orp_at_end() {
        assert_eq!(compute_word_offset(40, 1, 1), 39);
    }

    #[test]
    fn compute_offset_saturates_at_zero() {
        assert_eq!(compute_word_offset(2, 10, 1), 0);
    }

    #[test]
    fn compute_offset_zero_center() {
        assert_eq!(compute_word_offset(0, 0, 1), 0);
        assert_eq!(compute_word_offset(0, 5, 1), 0);
    }

    #[test]
    fn compute_offset_zoom2() {
        // At zoom 2, each char takes 2 cells. orp_index=2, center=40
        // offset = 40 - 2*2 = 36
        assert_eq!(compute_word_offset(40, 2, 2), 36);
    }

    #[test]
    fn compute_offset_zoom3() {
        // At zoom 3, each char takes 3 cells. orp_index=2, center=40
        // offset = 40 - 2*3 = 34
        assert_eq!(compute_word_offset(40, 2, 3), 34);
    }

    #[test]
    fn build_orp_line_simple_word() {
        let line = build_orp_line("Hello", 1);
        let spans = line.spans;
        assert_eq!(spans.len(), 3);

        // Pre-ORP: "H"
        assert_eq!(spans[0].content, "H");
        // ORP: "e"
        assert_eq!(spans[1].content, "e");
        assert_eq!(spans[1].style.fg, Some(Color::LightGreen));
        // Post-ORP: "llo"
        assert_eq!(spans[2].content, "llo");
    }

    #[test]
    fn build_orp_line_orp_at_start() {
        let line = build_orp_line("Word", 0);
        let spans = line.spans;
        assert_eq!(spans[0].content, "");
        assert_eq!(spans[1].content, "W");
        assert_eq!(spans[2].content, "ord");
    }

    #[test]
    fn build_orp_line_orp_at_end() {
        let line = build_orp_line("Hi", 1);
        let spans = line.spans;
        assert_eq!(spans[0].content, "H");
        assert_eq!(spans[1].content, "i");
        assert_eq!(spans[2].content, "");
    }

    #[test]
    fn build_orp_line_single_char() {
        let line = build_orp_line("I", 0);
        let spans = line.spans;
        assert_eq!(spans[0].content, "");
        assert_eq!(spans[1].content, "I");
        assert_eq!(spans[2].content, "");
    }

    #[test]
    fn build_orp_line_clamps_out_of_bounds_orp() {
        // ORP index beyond word length should be clamped
        let line = build_orp_line("Hi", 10);
        let spans = line.spans;
        // Clamped to index 1 (last char)
        assert_eq!(spans[0].content, "H");
        assert_eq!(spans[1].content, "i");
        assert_eq!(spans[2].content, "");
    }

    #[test]
    fn offset_alignment_various_words() {
        let center = 39;
        assert_eq!(compute_word_offset(center, 0, 1), 39);
        assert_eq!(compute_word_offset(center, 1, 1), 38);
        assert_eq!(compute_word_offset(center, 3, 1), 36);
    }

    #[test]
    fn build_padded_word_zoom1_no_spacing() {
        let line = build_padded_word_line("Hello", 1, 5, 1);
        let spans = &line.spans;
        assert_eq!(spans[0].content, "     "); // 5 spaces padding
        assert_eq!(spans[1].content, "H");     // pre-ORP
        assert_eq!(spans[2].content, "e");     // ORP
        assert_eq!(spans[3].content, "llo");   // post-ORP
    }

    #[test]
    fn build_padded_word_zoom2_spaced() {
        let line = build_padded_word_line("Hello", 1, 5, 2);
        let spans = &line.spans;
        assert_eq!(spans[0].content, "     ");   // padding
        assert_eq!(spans[1].content, "H ");      // "H" + 1 space before ORP
        assert_eq!(spans[2].content, "e");        // ORP char
        assert_eq!(spans[3].content, " l l o");   // space + l + space + l + space + o
    }

    #[test]
    fn build_padded_word_zoom3_extra_spaced() {
        let line = build_padded_word_line("Hi", 0, 5, 3);
        let spans = &line.spans;
        assert_eq!(spans[0].content, "     "); // padding
        assert_eq!(spans[1].content, "");      // no pre-ORP chars
        assert_eq!(spans[2].content, "H");     // ORP
        assert_eq!(spans[3].content, "  i");   // 2 spaces + i
    }
}
