# tapeworm

> TUI log file analyzer — tail, grep, aggregate, and visualize log patterns in real-time with histograms and heatmaps. Think htop for logs.

**Language:** Rust  
**Constraint:** The CLI tool that should exist  
**Stack:** ratatui, notify, regex, clap

---

## Features

- [x] **Phase 1:** Real-time tail with inotify/kqueue support
- [ ] **Phase 2:** Regex-based log parsing with user-defined profiles
- [ ] **Phase 3:** Histogram and heatmap visualizations in terminal
- [ ] **Phase 4:** Multi-file aggregation and cross-log correlation
- [ ] **Phase 5:** Export to JSON/CSV for further analysis
- [ ] **Phase 6:** Anomaly detection via rate spike detection

---

## Development Plan

1. ~~Phase 1: Core TUI scaffold with ratatui (file list, live tail view)~~ ✅
2. Phase 2: Log parsing engine with regex profiles (JSON config)
3. Phase 3: Aggregation engine — count, group by, time windows
4. Phase 4: Histogram and heatmap widgets
5. Phase 5: Anomaly detection (rate spikes, pattern breaks)
6. Phase 6: Export formats (JSON, CSV) and CLI args
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

---

## Architecture

### Phase 1: Core TUI Scaffold

The Phase 1 implementation provides:

- **`src/app.rs`** — Application state management (`App`, `FileState`, `Focus`)
- **`src/ui.rs`** — ratatui rendering (file list sidebar, tail view, status bar)
- **`src/tail.rs`** — File watching with `notify` crate + seek-based tail reading
- **`src/events.rs`** — Crossterm keyboard input handling
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
