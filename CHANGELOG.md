# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-06-10

### Added

- **Phase 1:** Real-time tail with inotify/kqueue support via the `notify` crate
- **Phase 2:** Regex-based log parsing with built-in and user-defined profiles (syslog, nginx, apache, json)
- **Phase 3:** Aggregation engine — count, group by, and time window bucketing (1m, 5m, 15m, 1h)
- **Phase 4:** Histogram and heatmap widgets for visualizing log patterns
- **Phase 5:** Anomaly detection — rate spikes, error spikes, level shifts, and new pattern detection
- **Phase 6:** Export formats — JSON and CSV output with custom paths
- **Phase 7:** Polish — 5 color themes (default, dark, light, solarized, monokai), keyboard controls, and man page
- Full CLI argument parsing with `clap` (tick-rate, initial-lines, profile, theme, export, no-parse)
- Comprehensive test suite with 82 unit tests covering all modules
- Man page at `tapeworm.1`

### Features

- Multi-file log watching with sidebar navigation
- Toggle between Tail, Aggregation, and Anomaly views at runtime
- Parsed vs raw line display toggle
- Real-time file watching with automatic refresh
- Built-in log level detection and color-coded output

[1.0.0]: https://github.com/tapeworm/tapeworm/releases/tag/v1.0.0
