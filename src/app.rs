use crate::aggregate::{AggregationEngine, AggregationResult};
use crate::parser::ParsedLine;
use crate::theme::Theme;
use std::path::PathBuf;

/// Application focus area
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    FileList,
    TailView,
}

/// Represents the state of a single watched file
#[derive(Debug, Clone)]
pub struct FileState {
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub parsed_lines: Vec<ParsedLine>,
    pub line_count: usize,
    pub parser_enabled: bool,
    pub aggregation: Option<AggregationResult>,
}

impl FileState {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lines: Vec::new(),
            parsed_lines: Vec::new(),
            line_count: 0,
            parser_enabled: true,
            aggregation: None,
        }
    }

    /// Recompute aggregation using the provided engine
    pub fn recompute_aggregation(&mut self, engine: &AggregationEngine) {
        self.aggregation = Some(engine.aggregate(&self.parsed_lines));
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.to_string_lossy().to_string())
    }

    pub fn push_lines(&mut self, new_lines: Vec<String>) {
        self.lines.extend(new_lines);
        // Keep only last 1000 lines to prevent unbounded growth
        if self.lines.len() > 1000 {
            self.lines.drain(0..self.lines.len() - 1000);
        }
        self.line_count = self.lines.len();
    }

    pub fn push_parsed_lines(&mut self, parsed: Vec<ParsedLine>) {
        self.parsed_lines.extend(parsed);
        // Keep only last 1000 parsed lines to prevent unbounded growth
        if self.parsed_lines.len() > 1000 {
            self.parsed_lines.drain(0..self.parsed_lines.len() - 1000);
        }
    }

    /// Get the effective lines to display - parsed if available, raw otherwise
    pub fn display_lines(&self) -> Vec<(String, Option<crate::parser::LogLevel>)> {
        if self.parser_enabled && !self.parsed_lines.is_empty() {
            self.parsed_lines
                .iter()
                .map(|p| (p.raw.clone(), p.level))
                .collect()
        } else {
            self.lines
                .iter()
                .map(|l| (l.clone(), None))
                .collect()
        }
    }
}

/// Available view modes for the main panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Tail,
    Aggregation,
}

/// Main application state
#[derive(Debug)]
pub struct App {
    pub files: Vec<FileState>,
    pub selected_index: usize,
    pub focus: Focus,
    pub should_quit: bool,
    pub tail_scroll: usize,
    pub show_parsed: bool,
    pub view_mode: ViewMode,
    pub aggregation_engine: AggregationEngine,
    pub theme: Theme,
}

impl App {
    pub fn new(file_paths: Vec<PathBuf>, theme: Theme) -> Self {
        let files = file_paths.into_iter().map(FileState::new).collect();
        Self {
            files,
            selected_index: 0,
            focus: Focus::FileList,
            should_quit: false,
            tail_scroll: 0,
            show_parsed: true,
            view_mode: ViewMode::Tail,
            aggregation_engine: AggregationEngine::default(),
            theme,
        }
    }

    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Tail => ViewMode::Aggregation,
            ViewMode::Aggregation => ViewMode::Tail,
        };
    }

    pub fn cycle_time_window(&mut self) {
        self.aggregation_engine.cycle_time_window();
        self.recompute_all_aggregations();
    }

    pub fn recompute_all_aggregations(&mut self) {
        for file in &mut self.files {
            file.recompute_aggregation(&self.aggregation_engine);
        }
    }

    pub fn selected_file(&self) -> Option<&FileState> {
        self.files.get(self.selected_index)
    }

    pub fn selected_file_mut(&mut self) -> Option<&mut FileState> {
        self.files.get_mut(self.selected_index)
    }

    pub fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.files.len();
            self.tail_scroll = 0;
        }
    }

    pub fn previous_file(&mut self) {
        if !self.files.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.files.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.tail_scroll = 0;
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::FileList => Focus::TailView,
            Focus::TailView => Focus::FileList,
        };
    }

    pub fn scroll_up(&mut self, amount: usize) {
        let max_scroll = self
            .selected_file()
            .map(|file| file.display_lines().len().saturating_sub(1))
            .unwrap_or(0);
        self.tail_scroll = self.tail_scroll.saturating_add(amount);
        if self.tail_scroll > max_scroll {
            self.tail_scroll = max_scroll;
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.tail_scroll = self.tail_scroll.saturating_sub(amount);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.tail_scroll = 0;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_parsed_view(&mut self) {
        self.show_parsed = !self.show_parsed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_app_new() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let app = App::new(paths, Theme::default());
        assert_eq!(app.files.len(), 1);
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.focus, Focus::FileList);
        assert!(!app.should_quit);
        assert!(app.show_parsed);
    }

    #[test]
    fn test_app_navigation() {
        let paths = vec![
            PathBuf::from("/tmp/a.log"),
            PathBuf::from("/tmp/b.log"),
            PathBuf::from("/tmp/c.log"),
        ];
        let mut app = App::new(paths, Theme::default());
        
        assert_eq!(app.selected_index, 0);
        app.next_file();
        assert_eq!(app.selected_index, 1);
        app.next_file();
        assert_eq!(app.selected_index, 2);
        app.next_file();
        assert_eq!(app.selected_index, 0);
        
        app.previous_file();
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn test_file_state() {
        let mut file = FileState::new(PathBuf::from("/tmp/test.log"));
        assert_eq!(file.name(), "test.log");
        
        file.push_lines(vec!["line1".to_string(), "line2".to_string()]);
        assert_eq!(file.lines.len(), 2);
        assert_eq!(file.line_count, 2);
    }

    #[test]
    fn test_file_state_display_lines_raw() {
        let mut file = FileState::new(PathBuf::from("/tmp/test.log"));
        file.push_lines(vec!["line1".to_string(), "line2".to_string()]);
        
        // When parser is disabled, should return raw lines
        file.parser_enabled = false;
        let display = file.display_lines();
        assert_eq!(display.len(), 2);
        assert_eq!(display[0].0, "line1");
        assert_eq!(display[0].1, None);
    }

    #[test]
    fn test_file_state_display_lines_parsed() {
        let mut file = FileState::new(PathBuf::from("/tmp/test.log"));
        file.push_lines(vec!["line1".to_string()]);
        
        // When parser is enabled and parsed lines exist, should return parsed
        let parsed = ParsedLine {
            raw: "parsed line".to_string(),
            fields: std::collections::HashMap::new(),
            level: Some(crate::parser::LogLevel::Error),
            timestamp: None,
        };
        file.push_parsed_lines(vec![parsed]);
        
        let display = file.display_lines();
        assert_eq!(display.len(), 1);
        assert_eq!(display[0].0, "parsed line");
        assert_eq!(display[0].1, Some(crate::parser::LogLevel::Error));
    }

    #[test]
    fn test_file_state_line_limit() {
        let mut file = FileState::new(PathBuf::from("/tmp/test.log"));
        let many_lines: Vec<String> = (0..1100).map(|i| format!("line{}", i)).collect();
        file.push_lines(many_lines);
        assert_eq!(file.lines.len(), 1000);
        assert_eq!(file.line_count, 1000);
    }

    #[test]
    fn test_focus_toggle() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        assert_eq!(app.focus, Focus::FileList);
        app.toggle_focus();
        assert_eq!(app.focus, Focus::TailView);
        app.toggle_focus();
        assert_eq!(app.focus, Focus::FileList);
    }

    #[test]
    fn test_scroll() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        // Add some lines
        if let Some(file) = app.selected_file_mut() {
            file.push_lines(vec!["line1".to_string(), "line2".to_string(), "line3".to_string()]);
        }
        
        app.scroll_up(1);
        assert_eq!(app.tail_scroll, 1);
        
        app.scroll_down(1);
        assert_eq!(app.tail_scroll, 0);
        
        app.scroll_to_bottom();
        assert_eq!(app.tail_scroll, 0);
    }

    #[test]
    fn test_toggle_parsed_view() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        assert!(app.show_parsed);
        app.toggle_parsed_view();
        assert!(!app.show_parsed);
        app.toggle_parsed_view();
        assert!(app.show_parsed);
    }

    #[test]
    fn test_cycle_theme() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        assert_eq!(app.theme, Theme::Default);
        app.cycle_theme();
        assert_eq!(app.theme, Theme::Dark);
        app.cycle_theme();
        assert_eq!(app.theme, Theme::Light);
    }
}
