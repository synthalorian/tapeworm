mod app;
mod events;
mod parser;
mod profile;
mod tail;
mod ui;

use app::App;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use events::{handle_key_event, EventHandler};
use parser::LogParser;
use profile::ProfileManager;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    io,
    path::PathBuf,
};
use tail::{TailEvent, TailWatcher};

/// tapeworm — TUI log file analyzer
#[derive(Parser, Debug)]
#[command(name = "tapeworm")]
#[command(about = "TUI log file analyzer — htop for logs")]
#[command(version)]
struct Cli {
    /// Log files to watch
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Tick rate in milliseconds (UI refresh interval)
    #[arg(short, long, default_value = "250")]
    tick_rate: u64,

    /// Number of initial lines to read from each file
    #[arg(short, long, default_value = "50")]
    initial_lines: usize,

    /// Log parsing profile to use (built-in: syslog, nginx, json) or path to custom JSON profile
    #[arg(short, long, default_value = "syslog")]
    profile: String,

    /// Disable log parsing (show raw lines only)
    #[arg(long)]
    no_parse: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Validate files exist
    for file in &cli.files {
        if !file.exists() {
            eprintln!("Error: File '{}' does not exist", file.display());
            std::process::exit(1);
        }
        if !file.is_file() {
            eprintln!("Error: '{}' is not a file", file.display());
            std::process::exit(1);
        }
    }

    // Setup profile manager and load selected profile
    let profile_manager = ProfileManager::new();
    let parser = if cli.no_parse {
        None
    } else {
        // Try to load as built-in profile first, then as file path
        let profile = if let Some(profile) = profile_manager.get(&cli.profile) {
            Some(profile.clone())
        } else {
            let profile_path = PathBuf::from(&cli.profile);
            if profile_path.exists() {
                match profile::LogProfile::from_file(&profile_path) {
                    Ok(profile) => Some(profile),
                    Err(e) => {
                        eprintln!("Warning: Failed to load profile '{}': {}", cli.profile, e);
                        None
                    }
                }
            } else {
                eprintln!("Warning: Unknown profile '{}', using raw lines", cli.profile);
                None
            }
        };

        profile.and_then(|p| match LogParser::new(p) {
            Ok(parser) => Some(parser),
            Err(e) => {
                eprintln!("Warning: Failed to compile profile regex: {}", e);
                None
            }
        })
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, cli, parser);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {:?}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    cli: Cli,
    parser: Option<LogParser>,
) -> io::Result<()> {
    let mut app = App::new(cli.files.clone());
    let event_handler = EventHandler::new(cli.tick_rate);
    let mut tail_watcher = TailWatcher::new().map_err(|e| {
        io::Error::new(io::ErrorKind::Other, format!("Failed to create tail watcher: {}", e))
    })?;

    // Watch all files and read initial lines
    for file_path in &cli.files {
        tail_watcher.watch(file_path).map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to watch '{}': {}", file_path.display(), e),
            )
        })?;

        // Read initial lines
        match TailWatcher::read_initial_lines(file_path, cli.initial_lines) {
            Ok(lines) => {
                if let Some(file_state) = app
                    .files
                    .iter_mut()
                    .find(|f| f.path == *file_path)
                {
                    // Parse initial lines if parser is available
                    if let Some(ref p) = parser {
                        let parsed = p.parse_lines(&lines);
                        file_state.push_parsed_lines(parsed);
                    }
                    file_state.push_lines(lines);
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not read initial lines from '{}': {}", file_path.display(), e);
            }
        }
    }

    // Main event loop
    while !app.should_quit {
        // Draw UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Handle events
        if let Some(event) = event_handler.next_event() {
            match event {
                events::AppEvent::Tick => {
                    // Poll for file changes
                    let tail_events = tail_watcher.poll_events();
                    for tail_event in tail_events {
                        match tail_event {
                            TailEvent::LinesAdded { path, lines } => {
                                if let Some(file_state) = app
                                    .files
                                    .iter_mut()
                                    .find(|f| f.path == path)
                                {
                                    // Parse new lines if parser is available
                                    if let Some(ref p) = parser {
                                        let parsed = p.parse_lines(&lines);
                                        file_state.push_parsed_lines(parsed);
                                    }
                                    file_state.push_lines(lines);
                                }
                            }
                            TailEvent::FileRemoved { path } => {
                                // File was removed, we keep the existing lines
                                // but could mark it as stale in the UI
                                let _ = path;
                            }
                        }
                    }
                }
                events::AppEvent::Key(key) => {
                    handle_key_event(&mut app, key);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse() {
        let cli = Cli::parse_from(["tapeworm", "/tmp/test.log"]);
        assert_eq!(cli.files.len(), 1);
        assert_eq!(cli.tick_rate, 250);
        assert_eq!(cli.initial_lines, 50);
        assert_eq!(cli.profile, "syslog");
        assert!(!cli.no_parse);
    }

    #[test]
    fn test_cli_parse_with_options() {
        let cli = Cli::parse_from([
            "tapeworm",
            "--tick-rate",
            "100",
            "--initial-lines",
            "20",
            "--profile",
            "nginx",
            "/tmp/test.log",
        ]);
        assert_eq!(cli.tick_rate, 100);
        assert_eq!(cli.initial_lines, 20);
        assert_eq!(cli.profile, "nginx");
    }

    #[test]
    fn test_cli_parse_no_parse() {
        let cli = Cli::parse_from([
            "tapeworm",
            "--no-parse",
            "/tmp/test.log",
        ]);
        assert!(cli.no_parse);
    }

    #[test]
    fn test_profile_manager_builtins() {
        let manager = ProfileManager::new();
        assert!(manager.get("syslog").is_some());
        assert!(manager.get("nginx").is_some());
        assert!(manager.get("json").is_some());
    }
}
