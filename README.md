# tapeworm

> TUI log file analyzer — tail, grep, aggregate, and visualize log patterns in real-time with histograms and heatmaps. Think htop for logs.

**Language:** Rust  
**Constraint:** The CLI tool that should exist  
**Stack:** ratatui, notify, regex, clap

---

## Features

- [x] **Phase 1:** Real-time tail with inotify/kqueue support
- [x] **Phase 2:** Regex-based log parsing with user-defined profiles
- [x] **Phase 3:** Aggregation engine — count, group by, time windows
- [x] **Phase 4:** Histogram and heatmap widgets
- [x] **Phase 5:** Anomaly detection (rate spikes, pattern breaks)
- [x] **Phase 6:** Export formats (JSON, CSV) and CLI args

---

## Development Plan

1. ~~Phase 1: Core TUI scaffold with ratatui (file list, live tail view)~~ ✅
2. ~~Phase 2: Log parsing engine with regex profiles (JSON config)~~ ✅
3. ~~Phase 3: Aggregation engine — count, group by, time windows~~ ✅
4. ~~Phase 4: Histogram and heatmap widgets~~ ✅
5. ~~Phase 5: Anomaly detection (rate spikes, pattern breaks)~~ ✅
6. ~~Phase 6: Export formats (JSON, CSV) and CLI args~~ ✅
7. Phase 7: Polish — themes, keybinds, man page

---

## Getting Started

### Prerequisites

- Rust toolchain

### Build

```bash
cargo build --release
```

### Run

```bash
# Watch a single log file
cargo run -- /var/log/syslog

# Watch multiple log files
cargo run -- /var/log/syslog /var/log/auth.log /var/log/nginx/access.log

# Adjust tick rate (UI refresh interval in ms)
cargo run -- --tick-rate 100 /var/log/syslog

# Adjust initial lines read from each file
cargo run -- --initial-lines 100 /var/log/syslog

# Export to JSON on exit
cargo run -- --export json /var/log/syslog

# Export to CSV with custom output path
cargo run -- --export csv --export-output /tmp/my-export.csv /var/log/syslog

# Use a custom log parsing profile
cargo run -- --profile nginx /var/log/nginx/access.log

# Disable parsing (raw lines only)
cargo run -- --no-parse /var/log/syslog

# Use a different color theme
cargo run -- --theme monokai /var/log/syslog
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |
| `Tab` | Switch focus between file list and tail view |
| `j` / `↓` | Next file (in file list) / Scroll up (in tail view) |
| `k` / `↑` | Previous file (in file list) / Scroll down (in tail view) |
| `G` | Scroll to bottom (in tail view) |
| `g` | Scroll to top (in tail view) |
| `a` | Toggle view mode (Tail → Aggregation → Anomaly) |
| `t` | Cycle time window for aggregation (1m → 5m → 15m → 1h) |
| `T` | Cycle color theme |
| `p` | Toggle parsed/raw view |

---

## Architecture

### Module Overview

- **`src/app.rs`** — Application state management (`App`, `FileState`, `Focus`, `ViewMode`)
- **`src/ui.rs`** — ratatui rendering (file list sidebar, tail view, aggregation view, anomaly view, status bar)
- **`src/tail.rs`** — File watching with `notify` crate + seek-based tail reading
- **`src/events.rs`** — Crossterm keyboard input handling
- **`src/parser.rs`** — Log line parsing with regex profiles and log level detection
- **`src/profile.rs`** — Built-in and custom JSON profile management
- **`src/aggregate.rs`** — Aggregation engine with time bucketing and group-by
- **`src/anomaly.rs`** — Anomaly detection (rate spikes, error spikes, new patterns)
- **`src/export.rs`** — Export to JSON and CSV formats
- **`src/theme.rs`** — Color theme management
- **`src/main.rs`** — CLI argument parsing with `clap`, terminal setup, main event loop

### Data Flow

```
[Log Files] → [notify watcher] → [TailWatcher::poll_events] → [App::push_lines] → [UI render]
                                    ↑
[Crossterm events] → [EventHandler::next_event] → [handle_key_event] → [App state mutations]
```

---

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture
```

---

## License

MIT
