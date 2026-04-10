use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Character width in the bitmap font (columns).
pub const CHAR_WIDTH: usize = 5;
/// Character height in the bitmap font (rows).
pub const CHAR_HEIGHT: usize = 5;
/// Spacing columns between characters.
pub const CHAR_SPACING: usize = 1;

/// Return the 5-row bitmap for a character. Each u8 is a 5-bit row
/// where bit 4 (0b10000) is the leftmost pixel.
/// NOTE: Only Latin characters (A-Z, 0-9) and basic punctuation are supported.
/// Non-Latin scripts (CJK, Cyrillic, Arabic, etc.) render as a box placeholder.
pub fn char_bitmap(c: char) -> [u8; 5] {
    match c {
        'A' | 'a' => [0b01110, 0b10001, 0b11111, 0b10001, 0b10001],
        'B' | 'b' => [0b11110, 0b10001, 0b11110, 0b10001, 0b11110],
        'C' | 'c' => [0b01111, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' | 'd' => [0b11110, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' | 'e' => [0b11111, 0b10000, 0b11110, 0b10000, 0b11111],
        'F' | 'f' => [0b11111, 0b10000, 0b11110, 0b10000, 0b10000],
        'G' | 'g' => [0b01111, 0b10000, 0b10011, 0b10001, 0b01110],
        'H' | 'h' => [0b10001, 0b10001, 0b11111, 0b10001, 0b10001],
        'I' | 'i' => [0b11111, 0b00100, 0b00100, 0b00100, 0b11111],
        'J' | 'j' => [0b00111, 0b00001, 0b00001, 0b10001, 0b01110],
        'K' | 'k' => [0b10001, 0b10010, 0b11100, 0b10010, 0b10001],
        'L' | 'l' => [0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' | 'm' => [0b10001, 0b11011, 0b10101, 0b10001, 0b10001],
        'N' | 'n' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001],
        'O' | 'o' => [0b01110, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' | 'p' => [0b11110, 0b10001, 0b11110, 0b10000, 0b10000],
        'Q' | 'q' => [0b01110, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' | 'r' => [0b11110, 0b10001, 0b11110, 0b10010, 0b10001],
        'S' | 's' => [0b01111, 0b10000, 0b01110, 0b00001, 0b11110],
        'T' | 't' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' | 'u' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' | 'v' => [0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' | 'w' => [0b10001, 0b10001, 0b10101, 0b11011, 0b10001],
        'X' | 'x' => [0b10001, 0b01010, 0b00100, 0b01010, 0b10001],
        'Y' | 'y' => [0b10001, 0b01010, 0b00100, 0b00100, 0b00100],
        'Z' | 'z' => [0b11111, 0b00010, 0b00100, 0b01000, 0b11111],
        '0' => [0b01110, 0b10011, 0b10101, 0b11001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b11111],
        '2' => [0b01110, 0b10001, 0b00110, 0b01000, 0b11111],
        '3' => [0b11110, 0b00001, 0b01110, 0b00001, 0b11110],
        '4' => [0b10010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b11110],
        '6' => [0b01110, 0b10000, 0b11110, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b00100],
        '8' => [0b01110, 0b10001, 0b01110, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b01111, 0b00001, 0b01110],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00100, 0b01000],
        '!' => [0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        '?' => [0b01110, 0b10001, 0b00110, 0b00000, 0b00100],
        '\'' => [0b00100, 0b00100, 0b00000, 0b00000, 0b00000],
        '"' => [0b01010, 0b01010, 0b00000, 0b00000, 0b00000],
        '-' => [0b00000, 0b00000, 0b11111, 0b00000, 0b00000],
        ':' => [0b00000, 0b00100, 0b00000, 0b00100, 0b00000],
        ';' => [0b00000, 0b00100, 0b00000, 0b00100, 0b01000],
        '(' => [0b00010, 0b00100, 0b00100, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00100, 0b00100, 0b01000],
        ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
        _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b11111], // box for unknown
    }
}

/// Render a single bitmap row for one character as a String of █ and spaces.
fn render_char_row(bitmap_row: u8) -> String {
    let mut s = String::with_capacity(CHAR_WIDTH);
    for bit in (0..CHAR_WIDTH).rev() {
        if bitmap_row & (1 << bit) != 0 {
            s.push('█');
        } else {
            s.push(' ');
        }
    }
    s
}

/// Compute the left padding so the ORP character's center column aligns to `center_x`.
pub fn compute_big_text_offset(center_x: usize, orp_index: usize) -> usize {
    let char_cell = CHAR_WIDTH + CHAR_SPACING;
    let orp_center = orp_index * char_cell + CHAR_WIDTH / 2;
    center_x.saturating_sub(orp_center)
}

/// Build CHAR_HEIGHT Lines for the word rendered in big block font.
/// The ORP character is highlighted in LightGreen; other characters are bold white.
pub fn render_big_word(text: &str, orp_index: usize, offset: usize) -> Vec<Line<'static>> {
    let word_style = Style::default().add_modifier(Modifier::BOLD);
    let orp_style = Style::default()
        .fg(Color::LightGreen)
        .add_modifier(Modifier::BOLD);

    let chars: Vec<char> = text.chars().collect();
    let clamped_orp = orp_index.min(chars.len().saturating_sub(1));

    let padding = " ".repeat(offset);
    let gap = " ".repeat(CHAR_SPACING);

    let mut lines = Vec::with_capacity(CHAR_HEIGHT);

    for row in 0..CHAR_HEIGHT {
        let mut spans: Vec<Span<'static>> = vec![Span::raw(padding.clone())];

        for (i, &ch) in chars.iter().enumerate() {
            let bitmap = char_bitmap(ch);
            let style = if i == clamped_orp { orp_style } else { word_style };
            spans.push(Span::styled(render_char_row(bitmap[row]), style));

            if i < chars.len() - 1 {
                spans.push(Span::raw(gap.clone()));
            }
        }

        lines.push(Line::from(spans));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_bitmap_space_is_blank() {
        let bm = char_bitmap(' ');
        assert!(bm.iter().all(|&row| row == 0));
    }

    #[test]
    fn char_bitmap_unknown_is_box() {
        let bm = char_bitmap('€');
        assert_eq!(bm[0], 0b11111);
        assert_eq!(bm[4], 0b11111);
    }

    #[test]
    fn render_char_row_full() {
        assert_eq!(render_char_row(0b11111), "█████");
    }

    #[test]
    fn render_char_row_empty() {
        assert_eq!(render_char_row(0b00000), "     ");
    }

    #[test]
    fn render_char_row_pattern() {
        assert_eq!(render_char_row(0b10101), "█ █ █");
    }

    #[test]
    fn compute_big_text_offset_basic() {
        // ORP at index 0, center=40 → offset = 40 - (0*6 + 2) = 38
        assert_eq!(compute_big_text_offset(40, 0), 38);
    }

    #[test]
    fn compute_big_text_offset_orp_in_middle() {
        // ORP at index 2, center=40 → offset = 40 - (2*6 + 2) = 26
        assert_eq!(compute_big_text_offset(40, 2), 26);
    }

    #[test]
    fn compute_big_text_offset_saturates() {
        assert_eq!(compute_big_text_offset(2, 10), 0);
    }

    #[test]
    fn render_big_word_returns_five_lines() {
        let lines = render_big_word("Hi", 0, 0);
        assert_eq!(lines.len(), CHAR_HEIGHT);
    }

    #[test]
    fn render_big_word_orp_styled() {
        let lines = render_big_word("AB", 0, 0);
        // First span on each line after padding should be the ORP char (A) with LightGreen
        let orp_style = Style::default()
            .fg(Color::LightGreen)
            .add_modifier(Modifier::BOLD);
        // spans[0] = padding (""), spans[1] = A bitmap row (orp), spans[2] = gap, spans[3] = B row
        assert_eq!(lines[0].spans[1].style, orp_style);
    }

    #[test]
    fn render_big_word_padding() {
        let lines = render_big_word("A", 0, 10);
        assert_eq!(lines[0].spans[0].content, "          "); // 10 spaces
    }

    #[test]
    fn case_insensitive_bitmaps() {
        assert_eq!(char_bitmap('a'), char_bitmap('A'));
        assert_eq!(char_bitmap('z'), char_bitmap('Z'));
    }
}
