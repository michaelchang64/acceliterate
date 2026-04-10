mod app;
mod core;
mod event;
mod ui;

use std::io;
use std::time::Duration;

use clap::Parser;
use color_eyre::eyre::{self, WrapErr};

/// Terminal speed reader
#[derive(Parser, Debug)]
#[command(name = "acceliterate", about = "Terminal speed reader")]
struct Args {
    /// Path to .txt file
    file: std::path::PathBuf,

    /// Initial words per minute
    #[arg(short, long, default_value_t = 300)]
    wpm: u32,

    /// Enable verbose logging to stderr
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> color_eyre::Result<()> {
    let args = Args::parse();

    color_eyre::install()?;

    if args.verbose {
        eprintln!("[acceliterate] verbose mode enabled");
    }

    // Read file
    let text = std::fs::read_to_string(&args.file)
        .wrap_err_with(|| format!("Failed to read file: {}", args.file.display()))?;

    // Tokenize
    let doc = core::tokenizer::tokenize(&text);

    if args.verbose {
        eprintln!(
            "[acceliterate] loaded {} words, {} paragraphs from {}",
            doc.total_words,
            doc.paragraphs.len(),
            args.file.display()
        );
    }

    // Validate document is not empty
    eyre::ensure!(!doc.words.is_empty(), "Document contains no words");

    // Create config and session
    let config = core::config::ReaderConfig { wpm: args.wpm };
    let session = core::reader::ReadingSession::new(doc, config);
    let mut app = app::App::new(session);

    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Install panic hook that restores terminal before printing panic
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        panic_hook(info);
    }));

    // Main loop
    loop {
        terminal.draw(|frame| ui::render(&app, frame))?;

        let timeout = if app.session.is_playing() {
            app.session.tick_duration()
        } else {
            Duration::from_millis(250)
        };

        if crossterm::event::poll(timeout)? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                event::handle_key(&mut app, key);
            }
        }

        if app.session.is_playing() {
            app.session.tick();
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_parses_file_arg() {
        let args = Args::try_parse_from(["acceliterate", "test.txt"]).unwrap();
        assert_eq!(args.file, std::path::PathBuf::from("test.txt"));
        assert_eq!(args.wpm, 300);
    }

    #[test]
    fn cli_parses_wpm_flag() {
        let args = Args::try_parse_from(["acceliterate", "--wpm", "450", "test.txt"]).unwrap();
        assert_eq!(args.wpm, 450);
    }

    #[test]
    fn cli_parses_wpm_short_flag() {
        let args = Args::try_parse_from(["acceliterate", "-w", "200", "test.txt"]).unwrap();
        assert_eq!(args.wpm, 200);
    }

    #[test]
    fn cli_requires_file_arg() {
        let result = Args::try_parse_from(["acceliterate"]);
        assert!(result.is_err());
    }

    #[test]
    fn cli_verbose_flag() {
        let args = Args::try_parse_from(["acceliterate", "-v", "test.txt"]).unwrap();
        assert!(args.verbose);
    }

    #[test]
    fn cli_verbose_default_false() {
        let args = Args::try_parse_from(["acceliterate", "test.txt"]).unwrap();
        assert!(!args.verbose);
    }
}
