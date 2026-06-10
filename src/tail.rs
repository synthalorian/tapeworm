use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TailError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Notify error: {0}")]
    Notify(#[from] notify::Error),
    #[error("Channel send error")]
    #[allow(dead_code)]
    SendError,
}

/// Event emitted when a file changes
#[derive(Debug, Clone)]
pub enum TailEvent {
    LinesAdded { path: PathBuf, lines: Vec<String> },
    FileRemoved { path: PathBuf },
}

/// Manages file tailing using notify for watching and manual seeking for reading
pub struct TailWatcher {
    watcher: RecommendedWatcher,
    file_positions: HashMap<PathBuf, u64>,
    rx: Receiver<notify::Result<Event>>,
}

impl TailWatcher {
    pub fn new() -> Result<Self, TailError> {
        let (tx, rx) = channel::<notify::Result<Event>>();
        let watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                let _ = tx.send(res);
            },
            Config::default(),
        )?;

        Ok(Self {
            watcher,
            file_positions: HashMap::new(),
            rx,
        })
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), TailError> {
        // Initialize file position at end of file
        if let Ok(metadata) = std::fs::metadata(path) {
            self.file_positions.insert(path.to_path_buf(), metadata.len());
        } else {
            self.file_positions.insert(path.to_path_buf(), 0);
        }

        self.watcher.watch(path, RecursiveMode::NonRecursive)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn unwatch(&mut self, path: &Path) -> Result<(), TailError> {
        self.file_positions.remove(path);
        self.watcher.unwatch(path)?;
        Ok(())
    }

    /// Poll for new tail events. Non-blocking.
    pub fn poll_events(&mut self) -> Vec<TailEvent> {
        let mut events = Vec::new();

        // Drain all available events from the channel
        while let Ok(result) = self.rx.try_recv() {
            match result {
                Ok(event) => {
                    if let Some(path) = event.paths.first() {
                        match event.kind {
                            notify::EventKind::Modify(_) | notify::EventKind::Create(_) => {
                                if let Some(new_lines) = self.read_new_lines(path) {
                                    if !new_lines.is_empty() {
                                        events.push(TailEvent::LinesAdded {
                                            path: path.to_path_buf(),
                                            lines: new_lines,
                                        });
                                    }
                                }
                            }
                            notify::EventKind::Remove(_) => {
                                events.push(TailEvent::FileRemoved {
                                    path: path.to_path_buf(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watch error: {:?}", e);
                }
            }
        }

        events
    }

    fn read_new_lines(&mut self, path: &Path) -> Option<Vec<String>> {
        let mut file = File::open(path).ok()?;
        let current_pos = *self.file_positions.get(path).unwrap_or(&0);

        // Get current file size
        let metadata = file.metadata().ok()?;
        let file_size = metadata.len();

        if file_size < current_pos {
            // File was truncated, start from beginning
            file.seek(SeekFrom::Start(0)).ok()?;
            self.file_positions.insert(path.to_path_buf(), 0);
        } else {
            file.seek(SeekFrom::Start(current_pos)).ok()?;
        }

        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut bytes_read = 0u64;

        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                bytes_read += line.len() as u64 + 1; // +1 for newline
                lines.push(line);
            }
        }

        // Update position
        let new_pos = if file_size < current_pos {
            bytes_read
        } else {
            current_pos + bytes_read
        };
        self.file_positions.insert(path.to_path_buf(), new_pos);

        if lines.is_empty() {
            None
        } else {
            Some(lines)
        }
    }

    /// Read initial content of a file (last N lines)
    pub fn read_initial_lines(path: &Path, max_lines: usize) -> Result<Vec<String>, TailError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

        if lines.len() > max_lines {
            lines = lines.split_off(lines.len() - max_lines);
        }

        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_tail_watcher_new() {
        let watcher = TailWatcher::new();
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_read_initial_lines() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line1").unwrap();
        writeln!(temp_file, "line2").unwrap();
        writeln!(temp_file, "line3").unwrap();

        let lines = TailWatcher::read_initial_lines(temp_file.path(), 2).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line2");
        assert_eq!(lines[1], "line3");
    }

    #[test]
    fn test_read_initial_lines_all() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "line1").unwrap();
        writeln!(temp_file, "line2").unwrap();

        let lines = TailWatcher::read_initial_lines(temp_file.path(), 10).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
    }
}
