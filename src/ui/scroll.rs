use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render highlighted scroll mode: full text with current word highlighted.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let playing = app.session.is_playing();
    let status_icon = if playing { " \u{25b6}" } else { " \u{23f8}" };
    let title = format!("Scroll{}", status_icon);

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let doc = &app.session.document;

    if doc.words.is_empty() {
        return;
    }

    // End of document.
    if app.session.position >= doc.words.len() {
        let end_msg = Paragraph::new(Line::from(Span::styled(
            "End of document",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        let y_offset = inner.height / 2;
        if y_offset < inner.height {
            let msg_area = Rect::new(inner.x, inner.y + y_offset, inner.width, 1);
            frame.render_widget(end_msg, msg_area);
        }
        return;
    }

    // Press space prompt.
    if !app.session.is_playing() && app.session.position == 0 && app.session.stats.words_read == 0 {
        let prompt = Paragraph::new(Line::from(Span::styled(
            "[ PRESS SPACE TO START ]",
            Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(ratatui::layout::Alignment::Center);
        let y_offset = inner.height / 2;
        if y_offset < inner.height {
            let msg_area = Rect::new(inner.x, inner.y + y_offset, inner.width, 1);
            frame.render_widget(prompt, msg_area);
        }
        return;
    }

    let pos = app.session.position;
    let width = inner.width as usize;

    // Current word's sentence for context highlighting.
    let (cur_para, cur_sent, _) = doc.word_positions[pos];

    let (lines, highlight_line) = build_scroll_lines(doc, pos, cur_para, cur_sent, width);

    // Scroll so highlighted line is roughly centered.
    let view_height = inner.height as usize;
    let scroll_y = highlight_line.saturating_sub(view_height / 2);

    let paragraph = Paragraph::new(lines).scroll((scroll_y as u16, 0));
    frame.render_widget(paragraph, inner);
}

/// Styles used in scroll mode.
struct ScrollStyles {
    highlight: Style,
    sentence: Style,
    dim: Style,
}

impl ScrollStyles {
    fn new() -> Self {
        Self {
            highlight: Style::default()
                .fg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            sentence: Style::default().fg(Color::White),
            dim: Style::default().fg(Color::DarkGray),
        }
    }

    fn for_word(&self, word_idx: usize, pos: usize, para: usize, sent: usize, wp: (usize, usize, usize)) -> Style {
        if word_idx == pos {
            self.highlight
        } else if wp.0 == para && wp.1 == sent {
            self.sentence
        } else {
            self.dim
        }
    }
}

/// Build wrapped lines from the document with styling applied.
/// Returns (lines, line_index_of_highlighted_word).
fn build_scroll_lines(
    doc: &crate::core::document::Document,
    pos: usize,
    cur_para: usize,
    cur_sent: usize,
    width: usize,
) -> (Vec<Line<'static>>, usize) {
    let styles = ScrollStyles::new();
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width: usize = 0;
    let mut highlight_line: usize = 0;

    for (i, word) in doc.words.iter().enumerate() {
        if word.is_paragraph_break {
            // Flush current line, then add a blank line.
            if !current_spans.is_empty() {
                lines.push(Line::from(std::mem::take(&mut current_spans)));
                current_width = 0;
            }
            lines.push(Line::from(""));
            continue;
        }

        let word_len = word.text.len();
        let needs_space = current_width > 0;
        let space_needed = if needs_space { word_len + 1 } else { word_len };

        // Wrap to next line if this word won't fit.
        if current_width + space_needed > width && current_width > 0 {
            lines.push(Line::from(std::mem::take(&mut current_spans)));
            current_width = 0;
        }

        let wp = doc.word_positions[i];
        let style = styles.for_word(i, pos, cur_para, cur_sent, wp);

        // Space before word.
        if current_width > 0 {
            // Space inherits the dimmer of the two adjacent styles.
            let space_style = if i == pos { styles.sentence } else { style };
            current_spans.push(Span::styled(" ", space_style));
            current_width += 1;
        }

        if i == pos {
            highlight_line = lines.len();
        }

        current_spans.push(Span::styled(word.text.clone(), style));
        current_width += word_len;
    }

    // Flush remaining spans.
    if !current_spans.is_empty() {
        lines.push(Line::from(current_spans));
    }

    (lines, highlight_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tokenizer::tokenize;

    #[test]
    fn build_scroll_lines_simple() {
        let doc = tokenize("Hello world.");
        let (lines, hl) = build_scroll_lines(&doc, 0, 0, 0, 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(hl, 0);
    }

    #[test]
    fn build_scroll_lines_wraps() {
        let doc = tokenize("Hello world today now.");
        // Width 12: "Hello world" = 11 fits, "today" wraps
        let (lines, _hl) = build_scroll_lines(&doc, 0, 0, 0, 12);
        assert!(lines.len() >= 2);
    }

    #[test]
    fn build_scroll_lines_paragraph_break() {
        let doc = tokenize("Hello.\n\nWorld.");
        let (lines, _hl) = build_scroll_lines(&doc, 0, 0, 0, 80);
        // "Hello." on one line, blank line, "World." on another = 3 lines
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn highlight_line_tracks_position() {
        let doc = tokenize("A B C D E F.");
        // Position 3 = "D"
        let (lines, hl) = build_scroll_lines(&doc, 3, 0, 0, 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(hl, 0); // all on one line
    }

    #[test]
    fn highlight_line_on_wrapped_line() {
        let doc = tokenize("AAAA BBBB CCCC DDDD.");
        // Width 10: "AAAA BBBB" = 9 fits on line 0, "CCCC" on line 1, "DDDD." on line 1 or 2
        let (lines, hl) = build_scroll_lines(&doc, 2, 0, 0, 10);
        // "CCCC" should be on line 1
        assert!(lines.len() >= 2);
        assert_eq!(hl, 1);
    }

    #[test]
    fn styles_highlight_current_word() {
        let doc = tokenize("Hello world.");
        let (lines, _) = build_scroll_lines(&doc, 0, 0, 0, 80);
        let spans = &lines[0].spans;
        // First span should be "Hello" with highlight style
        let hl_style = Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD);
        assert_eq!(spans[0].style, hl_style);
    }

    #[test]
    fn styles_dim_other_sentences() {
        let doc = tokenize("Hello. World.");
        // Position 0 = "Hello." (sentence 0). "World." is sentence 1 — should be dimmed.
        let (lines, _) = build_scroll_lines(&doc, 0, 0, 0, 80);
        let dim_style = Style::default().fg(Color::DarkGray);
        // Last span should be "World." in dim style
        let last = lines[0].spans.last().unwrap();
        assert_eq!(last.content, "World.");
        assert_eq!(last.style, dim_style);
    }

    #[test]
    fn empty_document() {
        let doc = tokenize("");
        let (lines, hl) = build_scroll_lines(&doc, 0, 0, 0, 80);
        assert_eq!(lines.len(), 0);
        assert_eq!(hl, 0);
    }
}
