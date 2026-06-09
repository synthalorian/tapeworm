use crate::app::{App, Focus};
use crate::parser::LogLevel;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render the full UI
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(f.size());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(chunks[0]);

    render_file_list(f, app, main_chunks[0]);
    render_tail_view(f, app, main_chunks[1]);
    render_status_bar(f, app, chunks[1]);
}

fn render_file_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let is_focused = app.focus == Focus::FileList;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = i == app.selected_index;
            let style = if is_selected {
                if is_focused {
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .bg(Color::DarkGray)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(Color::White)
            };

            let line_count = format!(" [{}]", file.line_count);
            let content = format!("{}{}", file.name(), line_count);
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(" Files ")
                .title_alignment(Alignment::Center),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

fn render_tail_view(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let is_focused = app.focus == Focus::TailView;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let title = if let Some(file) = app.selected_file() {
        let parse_indicator = if app.show_parsed && !file.parsed_lines.is_empty() {
            " [P]"
        } else {
            ""
        };
        format!(" {}{} ", file.name(), parse_indicator)
    } else {
        " No file selected ".to_string()
    };

    let text = if let Some(file) = app.selected_file() {
        let display_lines = file.display_lines();
        if display_lines.is_empty() {
            Text::from("Waiting for log lines...")
        } else {
            let start_idx = if app.tail_scroll > 0 {
                display_lines.len().saturating_sub(app.tail_scroll + area.height as usize)
            } else {
                display_lines.len().saturating_sub(area.height as usize)
            };
            let start_idx = start_idx.max(0);
            
            let visible_lines: Vec<Line> = display_lines[start_idx..]
                .iter()
                .map(|(line, level)| {
                    if app.show_parsed && level.is_some() {
                        let level_color = level_color(level.unwrap());
                        Line::from(vec![
                            Span::styled(
                                format!("[{}] ", level.unwrap().as_str()),
                                Style::default().fg(level_color).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(line.clone(), Style::default()),
                        ])
                    } else {
                        Line::from(line.clone())
                    }
                })
                .collect();
            
            Text::from(visible_lines)
        }
    } else {
        Text::from("Select a file to view its tail")
    };

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(title)
                .title_alignment(Alignment::Center),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let focus_text = match app.focus {
        Focus::FileList => "FILES",
        Focus::TailView => "TAIL",
    };

    let parse_indicator = if app.show_parsed { "P" } else { "R" };

    let help_text = format!(
        " [{}] q:quit | j/↓:down | k/↑:up | Tab:focus | G:bottom | p:parse({}) | ↑/↓:scroll ",
        focus_text, parse_indicator
    );

    let status = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(status, area);
}

/// Get the color for a log level
fn level_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Trace => Color::DarkGray,
        LogLevel::Debug => Color::Blue,
        LogLevel::Info => Color::Green,
        LogLevel::Warn => Color::Yellow,
        LogLevel::Error => Color::Red,
        LogLevel::Fatal => Color::Magenta,
        LogLevel::Unknown => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use std::path::PathBuf;

    #[test]
    fn test_render_app() {
        // This test mainly ensures the render functions don't panic
        let paths = vec![PathBuf::from("/tmp/test.log")];
        let app = App::new(paths);
        
        // We can't easily test rendering without a terminal, but we can verify
        // the app state is correct for rendering
        assert_eq!(app.focus, Focus::FileList);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_level_color() {
        assert_eq!(level_color(LogLevel::Trace), Color::DarkGray);
        assert_eq!(level_color(LogLevel::Debug), Color::Blue);
        assert_eq!(level_color(LogLevel::Info), Color::Green);
        assert_eq!(level_color(LogLevel::Warn), Color::Yellow);
        assert_eq!(level_color(LogLevel::Error), Color::Red);
        assert_eq!(level_color(LogLevel::Fatal), Color::Magenta);
        assert_eq!(level_color(LogLevel::Unknown), Color::White);
    }
}
