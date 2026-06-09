# tapeworm

> TUI log file analyzer — tail, grep, aggregate, and visualize log patterns in real-time with histograms and heatmaps. Think htop for logs.

**Language:** Rust  
**Constraint:** The CLI tool that should exist  
**Stack:** ratatui, notify, regex, clap

---

## Features

- Real-time tail with inotify/kqueue support
- Regex-based log parsing with user-defined profiles
- Histogram and heatmap visualizations in terminal
- Multi-file aggregation and cross-log correlation
- Export to JSON/CSV for further analysis
- Anomaly detection via rate spike detection

---

## Development Plan

1. Phase 1: Core TUI scaffold with ratatui (file list, live tail view)
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
# See PLAN.md for detailed build instructions per phase
cd tapeworm
```

### Run

```bash
# See PLAN.md for run instructions
```

---

## Architecture

See `PLAN.md` for detailed architecture decisions and implementation notes.

---

## License

MIT
