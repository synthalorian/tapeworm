use crate::app::{App, Focus};
use crate::theme::Theme;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    Tick,
    Key(KeyEvent),
}

/// Event handler that polls for crossterm events
pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Poll for the next event. Returns None if no event occurred within tick_rate.
    pub fn next_event(&self) -> Option<AppEvent> {
        if event::poll(self.tick_rate).ok()? {
            if let Ok(CrosstermEvent::Key(key)) = event::read() {
                return Some(AppEvent::Key(key));
            }
        }
        Some(AppEvent::Tick)
    }
}

/// Process a key event and update app state
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.quit();
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        KeyCode::Tab => {
            app.toggle_focus();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app.focus == Focus::FileList {
                app.next_file();
            } else {
                app.scroll_up(1);
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.focus == Focus::FileList {
                app.previous_file();
            } else {
                app.scroll_down(1);
            }
        }
        KeyCode::Char('G') => {
            if app.focus == Focus::TailView {
                app.scroll_to_bottom();
            }
        }
        KeyCode::Char('g') => {
            if app.focus == Focus::TailView {
                // Scroll to top - set scroll to max
                if let Some(file) = app.selected_file() {
                    app.tail_scroll = file.display_lines().len().saturating_sub(1);
                }
            }
        }
        KeyCode::Char('p') | KeyCode::Char('P') => {
            app.toggle_parsed_view();
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.toggle_view_mode();
        }
        KeyCode::Char('t') => {
            app.cycle_time_window();
        }
        KeyCode::Char('T') => {
            app.cycle_theme();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::path::PathBuf;

    fn make_key(code: KeyCode) -> KeyEvent {
        KeyEvent::from(code)
    }

    fn make_key_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_quit_key() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        handle_key_event(&mut app, make_key(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_ctrl_c_quit() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        handle_key_event(&mut app, make_key_with_modifiers(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }

    #[test]
    fn test_tab_focus() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        assert_eq!(app.focus, Focus::FileList);
        handle_key_event(&mut app, make_key(KeyCode::Tab));
        assert_eq!(app.focus, Focus::TailView);
    }

    #[test]
    fn test_navigation_keys() {
        let paths = vec![
            PathBuf::from("/tmp/a.log"),
            PathBuf::from("/tmp/b.log"),
        ];
        let mut app = App::new(paths, Theme::default());
        
        handle_key_event(&mut app, make_key(KeyCode::Char('j')));
        assert_eq!(app.selected_index, 1);
        
        handle_key_event(&mut app, make_key(KeyCode::Char('k')));
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_arrow_navigation() {
        let paths = vec![
            PathBuf::from("/tmp/a.log"),
            PathBuf::from("/tmp/b.log"),
        ];
        let mut app = App::new(paths, Theme::default());
        
        handle_key_event(&mut app, make_key(KeyCode::Down));
        assert_eq!(app.selected_index, 1);
        
        handle_key_event(&mut app, make_key(KeyCode::Up));
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_p_key_toggle_parsed() {
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let mut app = App::new(paths, Theme::default());
        
        assert!(app.show_parsed);
        handle_key_event(&mut app, make_key(KeyCode::Char('p')));
        assert!(!app.show_parsed);
        
        handle_key_event(&mut app, make_key(KeyCode::Char('P')));
        assert!(app.show_parsed);
    }
}
