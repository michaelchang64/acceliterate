use std::time::Duration;

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render the stats panel (WPM, progress, words read, time).
pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Stats");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let session = &app.session;
    let stats = &session.stats;

    let label_style = Style::default().fg(Color::Cyan);
    let value_style = Style::default().fg(Color::White);

    let current_wpm = format!("{}", session.wpm());
    let avg_wpm = format!("{:.0}", stats.average_wpm());
    let progress_pct = format!("{:.1}%", session.progress() * 100.0);
    let words_text = format!("{}/{}", stats.words_read, session.document.total_words);
    let time_text = format_duration(stats.elapsed_reading);
    let eta_text = estimate_remaining(
        session.document.total_words,
        stats.words_read,
        session.wpm(),
    );

    // Build the progress bar portion (compact, fits in the stats line).
    let progress_val = session.progress();
    let progress_bar = build_progress_bar(progress_val, 10);

    let line = Line::from(vec![
        Span::styled("WPM: ", label_style),
        Span::styled(current_wpm, value_style),
        Span::raw("  "),
        Span::styled("Avg: ", label_style),
        Span::styled(avg_wpm, value_style),
        Span::raw("  "),
        Span::styled("Progress: ", label_style),
        Span::styled(progress_pct, value_style),
        Span::raw(" "),
        Span::styled(progress_bar, value_style),
        Span::raw("  "),
        Span::styled("Words: ", label_style),
        Span::styled(words_text, value_style),
        Span::raw("  "),
        Span::styled("Time: ", label_style),
        Span::styled(time_text, value_style),
        Span::raw("  "),
        Span::styled("ETA: ", label_style),
        Span::styled(eta_text, value_style),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, inner);
}

/// Format a Duration as "m:ss" for display.
pub fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{}:{:02}", minutes, seconds)
}

/// Estimate remaining reading time based on words left and current WPM.
pub fn estimate_remaining(total_words: usize, words_read: usize, wpm: u32) -> String {
    let remaining = total_words.saturating_sub(words_read);
    if remaining == 0 || wpm == 0 {
        return "0:00".to_string();
    }
    let secs = (remaining as f64 / wpm as f64) * 60.0;
    format_duration(Duration::from_secs_f64(secs))
}

/// Build a compact text-based progress bar using block characters.
/// `progress` should be 0.0..=1.0, `width` is the character width of the bar.
pub fn build_progress_bar(progress: f64, width: usize) -> String {
    let clamped = progress.clamp(0.0, 1.0);
    let filled = (clamped * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0:00");
    }

    #[test]
    fn format_duration_seconds_only() {
        assert_eq!(format_duration(Duration::from_secs(5)), "0:05");
        assert_eq!(format_duration(Duration::from_secs(45)), "0:45");
    }

    #[test]
    fn format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1:00");
        assert_eq!(format_duration(Duration::from_secs(61)), "1:01");
        assert_eq!(format_duration(Duration::from_secs(272)), "4:32");
    }

    #[test]
    fn format_duration_large_value() {
        assert_eq!(format_duration(Duration::from_secs(3600)), "60:00");
        assert_eq!(format_duration(Duration::from_secs(3661)), "61:01");
    }

    #[test]
    fn format_duration_subsecond_truncated() {
        // Subsecond precision is dropped (we show m:ss)
        assert_eq!(format_duration(Duration::from_millis(1500)), "0:01");
        assert_eq!(format_duration(Duration::from_millis(59999)), "0:59");
    }

    #[test]
    fn progress_bar_empty() {
        let bar = build_progress_bar(0.0, 10);
        assert_eq!(bar, "[\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}]");
    }

    #[test]
    fn progress_bar_full() {
        let bar = build_progress_bar(1.0, 10);
        assert_eq!(bar, "[\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}]");
    }

    #[test]
    fn progress_bar_half() {
        let bar = build_progress_bar(0.5, 10);
        assert_eq!(bar, "[\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}]");
    }

    #[test]
    fn progress_bar_clamps_over_one() {
        let bar = build_progress_bar(1.5, 10);
        assert_eq!(bar, "[\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}]");
    }

    #[test]
    fn progress_bar_clamps_negative() {
        let bar = build_progress_bar(-0.5, 10);
        assert_eq!(bar, "[\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}\u{2591}]");
    }

    #[test]
    fn progress_bar_width_zero() {
        let bar = build_progress_bar(0.5, 0);
        assert_eq!(bar, "[]");
    }

    #[test]
    fn progress_bar_small_width() {
        let bar = build_progress_bar(0.5, 2);
        assert_eq!(bar, "[\u{2588}\u{2591}]");
    }

    #[test]
    fn estimate_remaining_full() {
        // 100 words remaining at 300 WPM = 20 seconds
        assert_eq!(estimate_remaining(200, 100, 300), "0:20");
    }

    #[test]
    fn estimate_remaining_done() {
        assert_eq!(estimate_remaining(100, 100, 300), "0:00");
    }

    #[test]
    fn estimate_remaining_zero_wpm() {
        assert_eq!(estimate_remaining(100, 0, 0), "0:00");
    }
}
