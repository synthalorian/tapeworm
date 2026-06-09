use crate::anomaly::AnomalySeverity;
use crate::app::{App, Focus, ViewMode};
use crate::parser::LogLevel;
use crate::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
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
    match app.view_mode {
        ViewMode::Tail => render_tail_view(f, app, main_chunks[1]),
        ViewMode::Aggregation => render_aggregation_view(f, app, main_chunks[1]),
        ViewMode::Anomaly => render_anomaly_view(f, app, main_chunks[1]),
    }
    render_status_bar(f, app, chunks[1]);
}

fn render_file_list(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let is_focused = app.focus == Focus::FileList;
    let theme = app.theme;
    let border_style = if is_focused {
        Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.secondary())
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
                        .bg(theme.selection_bg())
                        .fg(theme.selection_fg())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .bg(theme.secondary())
                        .fg(theme.text())
                        .add_modifier(Modifier::BOLD)
                }
            } else {
                Style::default().fg(theme.text())
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
    let theme = app.theme;
    let border_style = if is_focused {
        Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.secondary())
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
                        let level_color = level_color(level.unwrap(), &theme);
                        Line::from(vec![
                            Span::styled(
                                format!("[{}] ", level.unwrap().as_str()),
                                Style::default().fg(level_color).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(line.clone(), Style::default().fg(theme.text())),
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

fn render_aggregation_view(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let is_focused = app.focus == Focus::TailView;
    let theme = app.theme;
    let border_style = if is_focused {
        Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.secondary())
    };

    let title = if let Some(file) = app.selected_file() {
        format!(" Aggregation — {} [{}] ", file.name(), app.aggregation_engine.time_window.as_str())
    } else {
        " Aggregation — No file selected ".to_string()
    };

    let text = if let Some(file) = app.selected_file() {
        if let Some(ref agg) = file.aggregation {
            let mut lines: Vec<Line> = Vec::new();

            lines.push(Line::from(vec![
                Span::styled("Total Lines: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}", agg.total_count), Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(""));

            if !agg.by_level.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("By Level", Style::default().add_modifier(Modifier::BOLD).underlined()),
                ]));
                let level_counts = app.aggregation_engine.sorted_level_counts(agg);
                for (level, count) in level_counts {
                    let pct = if agg.total_count > 0 {
                        (count as f64 / agg.total_count as f64 * 100.0) as u64
                    } else {
                        0
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {:>8}: ", level.as_str()),
                            Style::default().fg(level_color(level, &theme)),
                        ),
                        Span::styled(format!("{:>6} ", count), Style::default()),
                        Span::styled(format!("({}%)", pct), Style::default().fg(Color::DarkGray)),
                    ]));
                }
                lines.push(Line::from(""));
            }

            if !agg.by_field.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Group By", Style::default().add_modifier(Modifier::BOLD).underlined()),
                ]));
                for (field_name, values) in &agg.by_field {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}:", field_name), Style::default().add_modifier(Modifier::BOLD)),
                    ]));
                    let mut sorted_values: Vec<(&String, &usize)> = values.iter().collect();
                    sorted_values.sort_by(|a, b| b.1.cmp(a.1));
                    for (value, count) in sorted_values.iter().take(10) {
                        lines.push(Line::from(vec![
                            Span::styled(format!("    {:>20}: ", value), Style::default().fg(Color::Yellow)),
                            Span::styled(format!("{}", count), Style::default()),
                        ]));
                    }
                    if sorted_values.len() > 10 {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("    ... and {} more", sorted_values.len() - 10),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]));
                    }
                    lines.push(Line::from(""));
                }
            }

            if !agg.time_buckets.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Time Buckets", Style::default().add_modifier(Modifier::BOLD).underlined()),
                ]));
                let buckets = app.aggregation_engine.sorted_time_buckets(agg);
                for (bucket, count) in buckets.iter().take(20) {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}: ", bucket), Style::default().fg(Color::Green)),
                        Span::styled(format!("{}", count), Style::default()),
                    ]));
                }
                if buckets.len() > 20 {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  ... and {} more buckets", buckets.len() - 20),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                }
            }

            Text::from(lines)
        } else {
            Text::from("No aggregation data available. Toggle aggregation with 'a'.")
        }
    } else {
        Text::from("Select a file to view its aggregation")
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

fn render_anomaly_view(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let is_focused = app.focus == Focus::TailView;
    let theme = app.theme;
    let border_style = if is_focused {
        Style::default().fg(theme.primary()).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.secondary())
    };

    let title = if let Some(file) = app.selected_file() {
        let severity_summary = if file.anomalies.is_empty() {
            " [OK]".to_string()
        } else {
            let crit = file
                .anomalies
                .iter()
                .filter(|a| a.severity == AnomalySeverity::Critical)
                .count();
            let high = file
                .anomalies
                .iter()
                .filter(|a| a.severity == AnomalySeverity::High)
                .count();
            if crit > 0 {
                format!(" [CRIT:{}]", crit)
            } else if high > 0 {
                format!(" [HIGH:{}]", high)
            } else {
                format!(" [{}]", file.anomalies.len())
            }
        };
        format!(" Anomalies — {}{} ", file.name(), severity_summary)
    } else {
        " Anomalies — No file selected ".to_string()
    };

    let text = if let Some(file) = app.selected_file() {
        if file.anomalies.is_empty() {
            if file.lines.is_empty() {
                Text::from("Waiting for log lines...")
            } else {
                Text::from("No anomalies detected. Baseline established from historical data.")
            }
        } else {
            let mut lines: Vec<Line> = Vec::new();

            let severity_counts =
                crate::anomaly::AnomalyEngine::severity_counts(&file.anomalies);
            let mut summary_parts = vec![Span::styled(
                "Summary: ",
                Style::default().add_modifier(Modifier::BOLD),
            )];
            let severity_order = [
                (AnomalySeverity::Critical, theme.fatal_color()),
                (AnomalySeverity::High, theme.error_color()),
                (AnomalySeverity::Medium, theme.warn_color()),
                (AnomalySeverity::Low, theme.trace_color()),
            ];
            for (sev, color) in severity_order {
                if let Some(count) = severity_counts.get(&sev) {
                    summary_parts.push(Span::styled(
                        format!("{}:{} ", sev.as_str(), count),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ));
                }
            }
            lines.push(Line::from(summary_parts));
            lines.push(Line::from(""));

            let sorted = crate::anomaly::AnomalyEngine::sorted_by_severity(&file.anomalies);
            for anomaly in sorted.iter().take(50) {
                let severity_color = anomaly_severity_color(anomaly.severity, &theme);
                let type_color = match anomaly.anomaly_type {
                    crate::anomaly::AnomalyType::RateSpike => theme.count_color(),
                    crate::anomaly::AnomalyType::ErrorSpike => theme.error_color(),
                    crate::anomaly::AnomalyType::NewPattern => theme.field_color(),
                    crate::anomaly::AnomalyType::LevelShift => theme.warn_color(),
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("[{}] ", anomaly.severity.as_str()),
                        Style::default()
                            .fg(severity_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{}: ", anomaly.anomaly_type.as_str()),
                        Style::default().fg(type_color),
                    ),
                    Span::styled(anomaly.description.clone(), Style::default().fg(theme.text())),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "          ",
                        Style::default(),
                    ),
                    Span::styled(
                        format!(
                            "bucket: {} | value: {} | expected: ~{}",
                            anomaly.bucket, anomaly.value, anomaly.expected
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }

            if sorted.len() > 50 {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    format!("... and {} more anomalies", sorted.len() - 50),
                    Style::default().fg(Color::DarkGray),
                )]));
            }

            Text::from(lines)
        }
    } else {
        Text::from("Select a file to view anomalies")
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
        Focus::TailView => match app.view_mode {
            ViewMode::Tail => "TAIL",
            ViewMode::Aggregation => "AGG",
            ViewMode::Anomaly => "ANOM",
        },
    };

    let parse_indicator = if app.show_parsed { "P" } else { "R" };
    let view_indicator = match app.view_mode {
        ViewMode::Tail => "tail",
        ViewMode::Aggregation => "agg",
        ViewMode::Anomaly => "anom",
    };

    let help_text = format!(
        " [{}] q:quit | j/↓:down | k/↑:up | Tab:focus | a:{} | t:tw | T:theme | p:parse({}) | ↑/↓:scroll ",
        focus_text, view_indicator, parse_indicator
    );

    let status = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Black).bg(Color::White))
        .alignment(Alignment::Center);

    f.render_widget(status, area);
}

/// Get the color for a log level
fn level_color(level: LogLevel, theme: &Theme) -> Color {
    match level {
        LogLevel::Trace => theme.trace_color(),
        LogLevel::Debug => theme.debug_color(),
        LogLevel::Info => theme.info_color(),
        LogLevel::Warn => theme.warn_color(),
        LogLevel::Error => theme.error_color(),
        LogLevel::Fatal => theme.fatal_color(),
        LogLevel::Unknown => theme.unknown_color(),
    }
}

fn anomaly_severity_color(severity: AnomalySeverity, theme: &Theme) -> Color {
    match severity {
        AnomalySeverity::Critical => theme.fatal_color(),
        AnomalySeverity::High => theme.error_color(),
        AnomalySeverity::Medium => theme.warn_color(),
        AnomalySeverity::Low => theme.trace_color(),
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
        let app = App::new(paths, Theme::default());
        
        // We can't easily test rendering without a terminal, but we can verify
        // the app state is correct for rendering
        assert_eq!(app.focus, Focus::FileList);
        assert!(!app.should_quit);
    }

    #[test]
    fn test_level_color() {
        let theme = Theme::default();
        assert_eq!(level_color(LogLevel::Trace, &theme), Color::DarkGray);
        assert_eq!(level_color(LogLevel::Debug, &theme), Color::Blue);
        assert_eq!(level_color(LogLevel::Info, &theme), Color::Green);
        assert_eq!(level_color(LogLevel::Warn, &theme), Color::Yellow);
        assert_eq!(level_color(LogLevel::Error, &theme), Color::Red);
        assert_eq!(level_color(LogLevel::Fatal, &theme), Color::Magenta);
        assert_eq!(level_color(LogLevel::Unknown, &theme), Color::White);
    }
}
