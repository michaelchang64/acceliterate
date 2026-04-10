# Acceliterate

A speed reading TUI application built in Rust using ratatui.

## Project Overview

Terminal-based speed reading app with three reading modes: RSVP (single-word flash), highlighted scroll (paced reading through full text), and Focus (split view combining both). Supports ORP highlighting, variable timing, zoom/big text, and session stats.

## Tech Stack

- **Language**: Rust (edition 2024)
- **TUI Framework**: ratatui 0.30 + crossterm 0.29
- **Architecture**: TEA (The Elm Architecture) — Model/View/Update pattern
- **Error Handling**: color-eyre 0.6
- **CLI**: clap 4

## Key Design Decisions

- Three reading modes: RSVP, Scroll, Focus (Tab cycles between them)
- ORP (Optimal Recognition Point) highlighting — letter ~25% into word highlighted in LightGreen
- Variable timing: word display duration adjusts for word length, punctuation, paragraph breaks
- Big text mode: 5x5 bitmap block font (Latin characters only)
- Zoom mode: letter spacing (levels 1-3) for RSVP
- High-water mark tracking prevents double-counting words on re-read
- File format support: .txt (v0), .md/.pdf/.epub planned (v1)
- Default WPM: 300, adjustable 50-1000
- Synchronous main loop with `crossterm::event::poll()` timeout as word-advance timer (no async needed)

## Build & Run

```bash
# Build the project
cargo build

# Run with a text file (default 300 WPM)
cargo run -- myfile.txt

# Run with custom WPM
cargo run -- -w 450 myfile.txt

# Run with verbose logging to stderr
cargo run -- -v myfile.txt

# Quick check — compiles but doesn't build binary (fastest feedback loop)
cargo check
```

## Keybindings

| Key | Action |
|-----|--------|
| Space | Play / Pause |
| Left/Right | Jump sentence back/forward |
| Up/Down | Adjust WPM by 25 |
| 0 | Restart from beginning |
| Tab | Cycle mode: RSVP → Scroll → Focus |
| . / , | Zoom in/out (RSVP letter spacing) |
| b | Toggle big text mode (RSVP) |
| ? | Help overlay |
| q | Quit |

## Testing

```bash
# Run all tests (~189 tests)
cargo test

# Run tests for a specific module
cargo test core::orp
cargo test core::reader
cargo test core::tokenizer
cargo test ui::rsvp
cargo test ui::scroll
cargo test ui::focus
cargo test event

# Run a single test by name
cargo test test_name_here

# Run tests with output visible (useful for debugging)
cargo test -- --nocapture
```

## Project Structure

```
src/
  main.rs           — CLI (clap), terminal setup/teardown, main loop
  app.rs            — App struct (ReadingSession + UI state), ReadingMode enum
  event.rs          — Keyboard event dispatch (key → action)
  core/             — Reading engine (no TUI deps)
    reader.rs       — ReadingSession state machine (play/pause/tick/navigate/restart)
    document.rs     — Word/Sentence/Paragraph/Document data structures
    tokenizer.rs    — Plain text → Document parser
    timing.rs       — Variable word display duration
    orp.rs          — Optimal Recognition Point calculation
    stats.rs        — Session statistics tracking (words, time, avg WPM)
    config.rs       — Reader configuration (WPM)
  ui/               — TUI rendering (ratatui)
    mod.rs          — Layout: splits screen into stats/reader/controls, mode dispatch
    rsvp.rs         — RSVP word display with ORP highlighting + redicle markers
    scroll.rs       — Highlighted scroll mode (full text, current word highlighted)
    focus.rs        — Focus mode (ORP word strip + scroll context split pane)
    bigfont.rs      — 5x5 bitmap block font (Latin A-Z, 0-9, punctuation)
    stats.rs        — Stats panel (WPM, progress bar, words, time, ETA)
    controls.rs     — Bottom keybinding hints bar
    help.rs         — Full-screen help overlay
```

## Conventions

- Keep modules focused and small — one clear purpose per file
- Follow TEA pattern: app state in `app.rs`, rendering in `ui/`, event handling in `event.rs`
- Core (`src/core/`) must never import from TUI (`src/ui/`) — strict dependency direction
- Use `color-eyre` for all error handling
- Prefer `thiserror` for custom error types
- All spec diagrams must be generated as SVG files (stored in `docs/superpowers/specs/diagrams/`)
