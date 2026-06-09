# tapeworm — Implementation Plan

## Project Overview

TUI log file analyzer — tail, grep, aggregate, and visualize log patterns in real-time with histograms and heatmaps. Think htop for logs.

**Language:** Rust  
**Constraint:** The CLI tool that should exist  
**Stack:** ratatui, notify, regex, clap

---

## Phase Breakdown

### Phase 1: Core TUI scaffold with ratatui (file list, live tail view)

**Goal:** Phase 1: Core TUI scaffold with ratatui (file list, live tail view)

**Deliverables:**
- [ ] Core implementation
- [ ] Tests
- [ ] Documentation update

**Notes:**
- 

---

### Phase 2: Log parsing engine with regex profiles (JSON config)

**Goal:** Phase 2: Log parsing engine with regex profiles (JSON config)

**Deliverables:**
- [ ] Core implementation
- [ ] Tests
- [ ] Documentation update

**Notes:**
- 

---

### Phase 3: Aggregation engine — count, group by, time windows

**Goal:** Phase 3: Aggregation engine — count, group by, time windows

**Deliverables:**
- [ ] Core implementation
- [ ] Tests
- [ ] Documentation update

**Notes:**
- 

---

### Phase 4: Histogram and heatmap widgets

**Goal:** Phase 4: Histogram and heatmap widgets

**Deliverables:**
- [ ] Core implementation
- [ ] Tests
- [ ] Documentation update

**Notes:**
- 

---

### Phase 5: Anomaly detection (rate spikes, pattern breaks)

**Goal:** Phase 5: Anomaly detection (rate spikes, pattern breaks)

**Deliverables:**
- [ ] Core implementation
- [ ] Tests
- [ ] Documentation update

**Notes:**
- 

---

### Phase 6: Export formats (JSON, CSV) and CLI args

**Goal:** Phase 6: Export formats (JSON, CSV) and CLI args

**Deliverables:**
- [x] Core implementation
- [x] Tests
- [x] Documentation update

**Notes:**
- Added `ExportFormat` enum supporting JSON and CSV
- Added `--export` and `--export-output` CLI arguments
- JSON export includes file metadata, parsed lines, and aggregation results
- CSV export produces flat tabular output with dynamic field columns
- Export triggered automatically on app exit when `--export` is provided 

---

### Phase 7: Polish — themes, keybinds, man page

**Goal:** Phase 7: Polish — themes, keybinds, man page

**Deliverables:**
- [x] Core implementation
- [x] Tests
- [x] Documentation update

**Notes:**
- Status bar and aggregation/anomaly views now use theme colors instead of hardcoded values
- Added `?` / `h` keyboard shortcut to toggle a help popup with all keybindings
- Added `Esc` to close the help popup
- Created `tapeworm.1` man page covering CLI options, keyboard controls, profiles, and examples 

---

## Architecture Notes

### Key Decisions

- 

### Data Flow

```
[Input] → [Parse] → [Transform] → [Output]
```

### Error Handling Strategy

- 

---

## Testing Strategy

- Unit tests for core functions
- Integration tests for full pipeline
- Benchmarks for performance-critical paths

---

## Open Questions

1. 
2. 

---

*Generated for opencode sprint. Implement phase by phase. DO NOT RESEARCH. Build directly.*
