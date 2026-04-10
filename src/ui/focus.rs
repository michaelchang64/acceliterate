use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Number of context words to show before and after the current word.
const CONTEXT_WORDS: usize = 3;

/// Render Focus mode: focused word strip on top, scroll view below.
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Focus strip (fixed, top)
            Constraint::Min(3),   // Scroll context (fill, bottom)
        ])
        .split(area);

    render_focus_strip(app, frame, chunks[0]);
    super::scroll::render(app, frame, chunks[1]);
}

/// Render the focused word strip: current word with ORP highlighting, surrounded by context.
fn render_focus_strip(app: &App, frame: &mut Frame, area: Rect) {
    let playing = app.session.is_playing();
    let status_icon = if playing { " \u{25b6}" } else { " \u{23f8}" };
    let title = format!("Focus{}", status_icon);

    let block = Block::default().borders(Borders::ALL).title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let doc = &app.session.document;

    if doc.words.is_empty() || app.session.position >= doc.words.len() {
        return;
    }

    let pos = app.session.position;
    let word = &doc.words[pos];
    let center_x = inner.width as usize / 2;

    let line = build_centered_context_line(doc, pos, word.orp_index, center_x);

    let y = inner.y + inner.height / 2;
    if y < inner.y + inner.height {
        let line_area = Rect::new(inner.x, y, inner.width, 1);
        frame.render_widget(Paragraph::new(line), line_area);
    }
}

/// Build a centered Line with ORP-highlighted current word and surrounding context.
/// The current word's ORP character is aligned to `center_x` via left-padding.
fn build_centered_context_line(
    doc: &crate::core::document::Document,
    pos: usize,
    orp_index: usize,
    center_x: usize,
) -> Line<'static> {
    let orp_style = Style::default()
        .fg(Color::LightGreen)
        .add_modifier(Modifier::BOLD);
    let word_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let context_style = Style::default().fg(Color::DarkGray);
    let ellipsis_style = Style::default().fg(Color::Indexed(239));

    // Gather preceding context words (skip paragraph breaks).
    let mut prev_words: Vec<String> = Vec::new();
    let mut idx = pos;
    while prev_words.len() < CONTEXT_WORDS && idx > 0 {
        idx -= 1;
        if !doc.words[idx].is_paragraph_break {
            prev_words.push(doc.words[idx].text.clone());
        }
    }
    prev_words.reverse();
    let has_more_before = pos > 0 && prev_words.len() == CONTEXT_WORDS;

    // Gather following context words (skip paragraph breaks).
    let mut next_words: Vec<String> = Vec::new();
    let mut scan = pos + 1;
    while next_words.len() < CONTEXT_WORDS && scan < doc.words.len() {
        if !doc.words[scan].is_paragraph_break {
            next_words.push(doc.words[scan].text.clone());
        }
        scan += 1;
    }
    let has_more_after = scan < doc.words.len();

    // Calculate how many chars the left context takes up.
    let mut left_len: usize = 0;
    if has_more_before {
        left_len += 4; // "... "
    }
    for (i, w) in prev_words.iter().enumerate() {
        if i > 0 || has_more_before {
            // space between words (or after ellipsis for first word)
        }
        if i > 0 {
            left_len += 1; // space
        }
        left_len += w.len();
    }
    if !prev_words.is_empty() {
        left_len += 1; // space before current word
    }

    // The ORP char of the current word should be at center_x.
    // Position of ORP in the full line = left_len + orp_index
    // So we need padding = center_x - (left_len + orp_index)
    let orp_absolute = left_len + orp_index;
    let padding = center_x.saturating_sub(orp_absolute);

    let mut spans: Vec<Span<'static>> = Vec::new();

    // Left padding.
    if padding > 0 {
        spans.push(Span::raw(" ".repeat(padding)));
    }

    // Leading ellipsis.
    if has_more_before {
        spans.push(Span::styled("... ", ellipsis_style));
    }

    // Previous context words.
    for (i, w) in prev_words.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ", context_style));
        }
        spans.push(Span::styled(w.clone(), context_style));
    }

    // Space before current word.
    if !prev_words.is_empty() {
        spans.push(Span::raw(" "));
    }

    // Current word with ORP highlighting (same as RSVP).
    let text = &doc.words[pos].text;
    let chars: Vec<char> = text.chars().collect();
    let clamped_orp = orp_index.min(chars.len().saturating_sub(1));

    let pre: String = chars[..clamped_orp].iter().collect();
    let orp_char: String = chars[clamped_orp..=clamped_orp].iter().collect();
    let post: String = chars[clamped_orp + 1..].iter().collect();

    if !pre.is_empty() {
        spans.push(Span::styled(pre, word_style));
    }
    spans.push(Span::styled(orp_char, orp_style));
    if !post.is_empty() {
        spans.push(Span::styled(post, word_style));
    }

    // Following context words.
    for w in &next_words {
        spans.push(Span::styled(" ", context_style));
        spans.push(Span::styled(w.clone(), context_style));
    }

    // Trailing ellipsis.
    if has_more_after {
        spans.push(Span::styled(" ...", ellipsis_style));
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::tokenizer::tokenize;

    fn line_text(line: &Line) -> String {
        line.spans.iter().map(|s| s.content.to_string()).collect()
    }

    #[test]
    fn context_line_single_word() {
        let doc = tokenize("Hello");
        let line = build_centered_context_line(&doc, 0, doc.words[0].orp_index, 40);
        let text = line_text(&line);
        assert!(text.contains("Hello"));
    }

    #[test]
    fn context_line_middle_word() {
        let doc = tokenize("A B C D E F G");
        let line = build_centered_context_line(&doc, 3, doc.words[3].orp_index, 40);
        let text = line_text(&line);
        assert!(text.contains("D"));
        assert!(text.contains("C"));
    }

    #[test]
    fn context_line_at_start_no_ellipsis() {
        let doc = tokenize("A B C D E");
        let line = build_centered_context_line(&doc, 0, 0, 40);
        let text = line_text(&line);
        assert!(!text.trim_start().starts_with("..."));
    }

    #[test]
    fn context_line_at_end_no_trailing_ellipsis() {
        let doc = tokenize("A B C D E");
        let line = build_centered_context_line(&doc, 4, 0, 40);
        let text = line_text(&line);
        assert!(!text.ends_with("..."));
    }

    #[test]
    fn context_line_orp_highlighted() {
        let doc = tokenize("A B C");
        // "B" orp_index=0, so the ORP char is "B" itself
        let line = build_centered_context_line(&doc, 1, 0, 40);
        let orp_style = Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD);
        let b_span = line.spans.iter().find(|s| s.content == "B").unwrap();
        assert_eq!(b_span.style, orp_style);
    }

    #[test]
    fn context_line_skips_paragraph_breaks() {
        let doc = tokenize("A B.\n\nC D.");
        let line = build_centered_context_line(&doc, 3, 0, 40);
        let text = line_text(&line);
        assert!(text.contains("B."));
        assert!(text.contains("C"));
    }

    #[test]
    fn current_word_centered_at_orp() {
        let doc = tokenize("AA BB CC DD EE FF GG");
        // "DD" at pos 3, orp_index=0, center_x=40
        // Left context: "AA BB CC" = 8 chars + space = 9 chars before "DD"
        // Padding should place ORP (first char of "DD") at column 40
        // So padding = 40 - 9 - 0 = 31
        let line = build_centered_context_line(&doc, 3, 0, 40);
        // Count chars before the "D" ORP span
        let mut cols = 0;
        for span in &line.spans {
            if span.content == "D"
                && span.style
                    == (Style::default()
                        .fg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD))
            {
                break;
            }
            cols += span.content.len();
        }
        assert_eq!(cols, 40);
    }
}
