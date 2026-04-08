# Acceliterate — Design Spec (v0)

## Overview

A terminal-based speed reading application built in Rust using ratatui. v0 delivers the core RSVP reading engine with plain text file support, ORP highlighting, variable timing, and essential UI panels.

## Goals (v0)

- Deliver a working RSVP speed reader in the terminal
- Load plain text files via CLI argument
- ORP-based word positioning with variable timing
- Stats tracking (WPM, progress, time)
- Be usable, fast, and distraction-free

## Non-Goals (v0)

- Highlighted scroll reading mode (v1)
- Multi-format parsing: pdf, epub, markdown, clipboard (v1)
- Document library, file picker, section picker (v1)
- Heuristic chunking / section detection (v1)
- Config file persistence, bookmarks (v1)
- Training features: speed burst, guided sessions (v1)
- OCR for scanned PDFs (v1)
- LLM comprehension quizzes (v2)
- Web/URL fetching
- Cloud sync
- Multi-language UI (English only, though content can be any language)

---

## 1. Architecture

### 1.1 Separation: Core Library vs TUI Frontend

The project is split into two layers:

- **`spreader_core`** — a frontend-agnostic library containing all reading engine logic. No TUI dependencies. Could be consumed by a web UI, mobile app, or any other frontend.
- **`acceliterate`** — the terminal frontend. Depends on `spreader_core`. Handles rendering, input, and terminal lifecycle.

This split can live in a single crate initially using module boundaries (no need for a Cargo workspace yet), but the dependency direction is strict: **core never imports from tui**.

**v0 source tree:**

```
src/
  main.rs              -- terminal setup/teardown, main loop (TUI layer)
  app.rs               -- App struct (TUI state), maps core state to UI
  event.rs             -- keyboard event handling, maps keys → core actions

  core/                -- FRONTEND-AGNOSTIC (no ratatui, no crossterm)
    mod.rs             -- public API re-exports
    reader.rs          -- ReadingSession: the main engine (play, pause, advance, jump, set_wpm)
    document.rs        -- Document, Paragraph, Sentence, Word structs
    tokenizer.rs       -- text → Document: tokenization, ORP computation, timing calc
    timing.rs          -- variable timing logic (word length, punctuation multipliers)
    orp.rs             -- ORP index calculation, word alignment
    stats.rs           -- session stats tracking (words read, avg WPM, elapsed time)
    config.rs          -- reader config (WPM, chunk size)

  ui/                  -- TUI-SPECIFIC (ratatui + crossterm)
    mod.rs             -- top-level layout, delegates to sub-renderers
    rsvp.rs            -- RSVP word display with ORP highlighting + redicle
    stats.rs           -- stats panel (WPM, words read, time, progress)
    controls.rs        -- bottom control hints bar
    help.rs            -- full-screen help overlay
```

**Full source tree (v1+):** see [v1 spec](2026-04-08-v1-features-design.md) for additions including parsers, library, scroll mode, training, and LLM modules.

### 1.2 Core API Surface (v0)

The core exposes a `ReadingSession` that any frontend drives:

```rust
// Frontend-agnostic — no rendering, no input handling
pub struct ReadingSession { ... }

impl ReadingSession {
    pub fn new(document: Document, config: ReaderConfig) -> Self;

    // Playback control
    pub fn play(&mut self);
    pub fn pause(&mut self);
    pub fn toggle(&mut self);
    pub fn is_playing(&self) -> bool;

    // Advance/navigation
    pub fn tick(&mut self) -> bool;        // advance if enough time elapsed; returns true if word changed
    pub fn tick_duration(&self) -> Duration; // how long until next tick (frontend uses this for poll/setTimeout)
    pub fn jump_forward_sentence(&mut self);
    pub fn jump_back_sentence(&mut self);

    // Speed control
    pub fn set_wpm(&mut self, wpm: u32);
    pub fn adjust_wpm(&mut self, delta: i32);
    pub fn wpm(&self) -> u32;

    // Current state (frontend reads this to render)
    pub fn current_word(&self) -> &Word;
    pub fn current_chunk(&self) -> &[Word];  // when chunk_size > 1
    pub fn current_sentence(&self) -> &Sentence;
    pub fn current_paragraph(&self) -> &Paragraph;
    pub fn progress(&self) -> f64;           // 0.0 to 1.0
    pub fn stats(&self) -> &SessionStats;
}
```

A web frontend would call the same methods — only the rendering and event loop differ.

**v1 additions** to `ReadingSession`: training methods (`start_speed_burst`, `start_guided_session`, etc.), `set_reading_mode()`, `set_chunk_size()`. See [v1 spec](2026-04-08-v1-features-design.md).

### 1.3 TUI Pattern: TEA (The Elm Architecture)

The TUI layer follows Model/View/Update:

- **Model**: `App` struct wraps `ReadingSession` + UI-specific state (panel visibility, help overlay open, etc.)
- **View**: `ui/` module reads `&App` and renders with ratatui
- **Update**: `event.rs` maps key events to `ReadingSession` methods + UI state changes

### 1.4 Main Loop (synchronous, no async runtime)

```
1. Initialize terminal (enter alternate screen, enable raw mode)
2. Install panic hook via color-eyre (restores terminal on panic)
3. Loop:
   a. crossterm::event::poll(session.tick_duration())
   b. If event received → handle input, call session/app methods
   c. If tick elapsed + playing → session.tick()
   d. terminal.draw(|frame| ui::render(&app, frame))
   e. Check exit condition
4. Restore terminal (leave alternate screen, disable raw mode)
```

---

## 2. Reading Mode (v0: RSVP Only)

### 2.1 RSVP Mode

Words displayed one at a time at a fixed center point on screen.

**ORP (Optimal Recognition Point):**
- Highlight the letter at ~25% into the word in red (or configured accent color)
- Formula: `orp_index = min(4, floor(word_length * 0.25))`
- Word positioned so ORP letter aligns to a fixed screen column
- Redicle markers (▼ above, ▲ below) at the ORP column

**Variable timing:**

| Condition | Multiplier |
|-----------|------------|
| Word length 1-3 chars | 0.85x base delay |
| Word length 4-7 chars | 1.0x base delay |
| Word length 8-11 chars | 1.3x base delay |
| Word length 12+ chars | 1.6x base delay |
| Comma, semicolon, colon | +0.4x base delay |
| Period, !, ? | +1.0x base delay |
| Paragraph break | +2.0x base delay (blank frame) |

Base delay: `60_000 / wpm` milliseconds.

**v1 additions:** Configurable chunk size (1-3 words per display), highlighted scroll mode, mode switching. See [v1 spec](2026-04-08-v1-features-design.md).

---

## 3. UI Layout (v0)

The v0 layout is a simplified version of the full UI:

```
┌─────────────────────────────────────┐
│              Stats Panel            │  ← WPM, progress %, words read, time
├─────────────────────────────────────┤
│                                     │
│          RSVP Word Display          │  ← ORP-aligned word with redicle
│                                     │
├─────────────────────────────────────┤
│           Control Hints             │  ← keybinding reminders
└─────────────────────────────────────┘
```

**Panels:**
- Stats panel — always visible in v0
- Control hints bar — always visible in v0
- `?` — full help overlay (lists all keybindings)

**v1 additions:** Toggleable panels (`s`, `c`, `h`), context paragraph panel, zen mode. See full layout diagram in [v1 spec](2026-04-08-v1-features-design.md).

---

## 4. App Flow (v0)

v0 is CLI-argument driven — no library screen:

```
acceliterate <file.txt>  →  Parse text  →  Reader
```

- File path is a required CLI argument
- Plain text files only (.txt)
- On parse failure, print error and exit
- `q` quits the app

**v1 additions:** Document library screen, file picker with tab completion, section picker, heuristic chunking, bookmarks, multi-format support. See [v1 spec](2026-04-08-v1-features-design.md).

---

## 5. Keybindings (v0)

| Key | Action |
|-----|--------|
| Space | Play / Pause |
| Left arrow | Jump back 1 sentence |
| Right arrow | Jump forward 1 sentence |
| Up arrow | Increase WPM by 25 |
| Down arrow | Decrease WPM by 25 |
| `?` | Full help overlay |
| `q` | Quit |

**v1 additions:** `Tab` (mode switch), `b` (speed burst), `g` (guided training), `c`/`s`/`h` (panel toggles), `Ctrl+L` (log viewer).

---

## 6. Text Processing Pipeline (v0)

```
.txt file → Read bytes → Split paragraphs → Split sentences → Tokenize words → Compute Timing + ORP → Document
```

### 6.1 Plain Text Parsing

- Read file as UTF-8 string
- Split on double newlines → `Paragraph` boundaries
- Split paragraphs into sentences (period/!/? followed by whitespace or end)
- Split sentences into words on whitespace
- Each word gets ORP index and timing multipliers computed

**v1 additions:** `TextParser` trait, format detection, markdown/PDF/EPUB/clipboard parsers, OCR pipeline. See [v1 spec](2026-04-08-v1-features-design.md).

### 6.2 Word Struct

```rust
struct Word {
    text: String,
    orp_index: usize,
    base_delay_multiplier: f32,   // word length factor
    punctuation_delay_multiplier: f32, // punctuation factor
    is_paragraph_break: bool,
}
```

### 6.3 Navigation Structures

Words grouped into `Sentence` and `Paragraph` containers:
- Left/Right arrow jumps between sentences
- Progress calculated as word_index / total_words

---

## 7. Session Stats (v0)

Tracked throughout a reading session and displayed in the stats panel:
- Words read (count)
- Elapsed reading time (excludes pauses)
- Average WPM (calculated: words_read / elapsed_minutes)
- Current WPM setting
- Progress: words_read / total_words (percentage + progress bar)
- Estimated time remaining

**v1 additions:** Speed burst, guided training sessions, session history persistence. See [v1 spec](2026-04-08-v1-features-design.md).

---

## 8. CLI Arguments (v0)

```
acceliterate [OPTIONS] <FILE>

Arguments:
  <FILE>  Path to .txt file (required)

Options:
  -w, --wpm <WPM>       Initial words per minute (default: 300)
  -v, --verbose         Enable logging to stderr
```

No config file persistence in v0 — all settings via CLI args with sensible defaults.

**v1 additions:** `--clipboard`, `--chunk-size`, `--mode`, config file (`~/.config/acceliterate/config.toml`), bookmarks, session history. See [v1 spec](2026-04-08-v1-features-design.md).

---

## 9. Error Handling (v0)

- `color-eyre` for all error propagation and panic recovery
- Terminal restoration guaranteed via panic hook (installed before entering alternate screen)
- File not found / not readable → clear error message and exit
- Non-UTF-8 content → error message and exit

---

## 10. Dependencies (v0)

```toml
[dependencies]
ratatui = "0.30"
crossterm = "0.29"
color-eyre = "0.6"
clap = { version = "4", features = ["derive"] }
unicode-segmentation = "1"
```

Minimal dependency set. No serialization, no file format crates, no logging framework in v0.

**v1 additions:** `thiserror`, `serde`/`toml`, `arboard`, format-specific crates (pdf, epub, md), `tracing` stack. See [v1 spec](2026-04-08-v1-features-design.md).

---

## 11. Versioning / Scope Summary

| Version | Scope |
|---------|-------|
| **v0** | RSVP reader: txt file loading (CLI arg), ORP highlighting, variable timing, stats panel, help overlay, play/pause/navigate/WPM controls |
| **v1** | Highlighted scroll mode, multi-format parsing (md/pdf/epub/clipboard), document library + file picker + section picker, heuristic chunking, config persistence, bookmarks, training (speed burst, guided sessions), OCR, session history, logging mode, context panel, panel toggles |
| **v2** | LLM comprehension quiz mode — see [v2 spec](2026-04-08-v2-llm-quiz-mode-design.md) |

Full v1 spec: [v1-features-design.md](2026-04-08-v1-features-design.md)

---

## 12. Academic Research Basis

Full citations with DOIs available in `specs/research/speed-reading-literature.md`. Key papers informing this design:

- **Rayner et al. (2016)**: Speed-accuracy trade-off is fundamental; default 250-300 WPM is correct
- **O'Regan & Jacobs (1992)**: ORP at ~25% of word length is optimal for recognition
- **Castelhano & Muter (2001)**: Chunked RSVP (2-3 words) beats single-word for comprehension
- **Brysbaert (2019)**: Average adult reads at 238 WPM — our default of 300 is a reasonable stretch target
- **Acklin & Papesh (2017)**: RSVP works at moderate speeds; comprehension testing is essential
- **Benedetto et al. (2015)**: Session fatigue after ~20 minutes — consider break reminders
