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
cargo build
cargo run -- <file_path>
```

## Conventions

- Keep modules focused and small — one clear purpose per file
- Follow TEA pattern: app state in `app.rs`, rendering in `ui.rs`, event handling in `event.rs`
- Use `color-eyre` for all error handling
- Prefer `thiserror` for custom error types
- All spec diagrams must be generated as SVG files (stored in `docs/superpowers/specs/diagrams/`)
