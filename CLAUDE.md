# Acceliterate

A speed reading TUI application built in Rust using ratatui.

## Project Overview

Terminal-based speed reading app that supports RSVP (Rapid Serial Visual Presentation) and highlighted scroll reading modes. Training-focused with session stats, speed burst mode, and comprehension checks.

## Tech Stack

- **Language**: Rust (edition 2024)
- **TUI Framework**: ratatui + crossterm
- **Architecture**: TEA (The Elm Architecture) — Model/View/Update pattern
- **Error Handling**: color-eyre
- **CLI**: clap

## Key Design Decisions

- Two reading modes: RSVP (single-word/chunk flash) and highlighted scroll (paced reading through full text)
- ORP (Optimal Recognition Point) highlighting in RSVP mode — letter ~25% into word highlighted in contrasting color
- Variable timing: word display duration adjusts for word length, punctuation, paragraph breaks
- File format support: .txt, .md, .pdf, .epub + clipboard paste
- Default WPM: 250-300 for new users
- Synchronous main loop with `crossterm::event::poll()` timeout as word-advance timer (no async needed)

## Build & Run

```bash
# Build the project
cargo build

# Run with a text file (default 300 WPM)
cargo run -- myfile.txt

# Run with custom WPM
cargo run -- -w 450 myfile.txt

# Quick check — compiles but doesn't build binary (fastest feedback loop)
cargo check
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific module
cargo test core::orp
cargo test core::reader
cargo test core::tokenizer
cargo test ui::rsvp
cargo test event

# Run a single test by name
cargo test test_name_here

# Run tests with output visible (useful for debugging)
cargo test -- --nocapture

# Run only ignored tests (if any)
cargo test -- --ignored
```

## Project Structure

```
src/
  main.rs           — CLI (clap), terminal setup/teardown, main loop
  app.rs            — App struct: wraps ReadingSession + UI state
  event.rs          — Keyboard event dispatch (key → action)
  core/             — Reading engine (no TUI deps)
    reader.rs       — ReadingSession state machine (play/pause/tick/navigate)
    document.rs     — Word/Sentence/Paragraph/Document data structures
    tokenizer.rs    — Plain text → Document parser
    timing.rs       — Variable word display duration
    orp.rs          — Optimal Recognition Point calculation
    stats.rs        — Session statistics tracking
    config.rs       — Reader configuration
  ui/               — TUI rendering (ratatui)
    mod.rs          — Layout: splits screen into stats/rsvp/controls
    rsvp.rs         — RSVP word display with ORP highlighting
    stats.rs        — Stats panel (WPM, progress, time)
    controls.rs     — Bottom keybinding hints bar
    help.rs         — Full-screen help overlay
```

## Conventions

- Keep modules focused and small — one clear purpose per file
- Follow TEA pattern: app state in `app.rs`, rendering in `ui.rs`, event handling in `event.rs`
- Use `color-eyre` for all error handling
- Prefer `thiserror` for custom error types
- All spec diagrams must be generated as SVG files (stored in `docs/superpowers/specs/diagrams/`)
