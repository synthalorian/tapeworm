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
- [ ] Core implementation
- [ ] Tests
- [ ] Documentation update

**Notes:**
- 

---

### Phase 7: Polish — themes, keybinds, man page

**Goal:** Phase 7: Polish — themes, keybinds, man page

**Deliverables:**
- [ ] Core implementation
- [ ] Tests
- [ ] Documentation update

**Notes:**
- 

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
