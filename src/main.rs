use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use clap::Parser;

mod detector;
mod engine;
mod fixer;
mod github;
mod issue;
mod ui;

use detector::{run_detection, DetectionResult};
use engine::Engine;
use fixer::{run_fix_with_progress, FixResult, ProgressFixHandle};
use github::{search_issues, SearchResult};

#[derive(Parser)]
#[command(
    name = "omadoctor",
    version = "0.1.0",
    about = "Diagnose and fix common Omarchy Linux issues",
    long_about = "omadoctor is a CLI tool for Omarchy Linux that detects and fixes common system issues.

Run without arguments for an interactive TUI, or use --scan for non-interactive scanning."
)]
struct Cli {
    /// Run all detections non-interactively and report results
    #[arg(short, long)]
    scan: bool,
}

struct AppState {
    show_detail: bool,
    detection_result: Option<DetectionResult>,
    fix_result: Option<FixResult>,
    fixing_in_progress: bool,
    fix_progress: Option<i32>,
    fix_handle: Option<ProgressFixHandle>,
    search_result: Option<SearchResult>,
    searching: bool,
    show_confirm: bool,
    show_manual_commands: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            show_detail: false,
            detection_result: None,
            fix_result: None,
            fixing_in_progress: false,
            fix_progress: None,
            fix_handle: None,
            search_result: None,
            searching: false,
            show_confirm: false,
            show_manual_commands: false,
        }
    }

    fn clear_results(&mut self) {
        self.detection_result = None;
        self.fix_result = None;
        self.fixing_in_progress = false;
        self.fix_progress = None;
        self.fix_handle = None;
        self.search_result = None;
        self.searching = false;
        self.show_confirm = false;
        self.show_manual_commands = false;
    }
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    let mut engine = Engine::new();

    if cli.scan {
        return run_scan_mode(&engine);
    }

    // TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = AppState::new();
    let res = run_app(&mut terminal, &mut engine, &mut app_state);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_scan_mode(engine: &Engine) -> Result<(), io::Error> {
    println!("🔍 Scanning your system...\n");

    let mut issues_found = 0;

    for issue in &engine.issues {
        let detection = run_detection(&issue.detection);
        let icon = issue.display_name().chars().next().unwrap_or('🔧');

        if detection.issue_found {
            println!("❌ {}: {}", icon, issue.name);
            issues_found += 1;
        } else {
            println!("✅ {}: {}", icon, issue.name);
        }
    }

    println!();

    if issues_found > 0 {
        println!("{} issue(s) found. Run `omadoctor` for interactive fixes.", issues_found);
        std::process::exit(1);
    } else {
        println!("All systems operational. No issues found!");
        std::process::exit(0);
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, engine: &mut Engine, app_state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, engine, app_state))?;

        // Check for progress updates if fix is in progress
        if app_state.fixing_in_progress {
            if let Some(ref handle) = app_state.fix_handle {
                // Try to receive progress updates without blocking
                loop {
                    match handle.progress_receiver.try_recv() {
                        Ok(progress) => {
                            app_state.fix_progress = Some(progress);
                        }
                        Err(_) => break,
                    }
                }

                // Check if result is ready (when progress hits 100)
                if app_state.fix_progress == Some(100) {
                    // Try to get the final result
                    if let Ok(result) = handle.result_receiver.try_recv() {
                        app_state.fix_result = Some(result);
                        app_state.fixing_in_progress = false;
                        app_state.fix_handle = None;
                    }
                }
            }
        }

        // Use a shorter timeout for event polling when fixing to keep UI responsive
        let poll_timeout = if app_state.fixing_in_progress {
            std::time::Duration::from_millis(50)
        } else {
            std::time::Duration::from_millis(100)
        };

        if crossterm::event::poll(poll_timeout)? {
            if let Event::Key(key) = event::read()? {
                // Handle confirmation dialog keys first
                if app_state.show_confirm {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // Yes - start the fix with progress
                            app_state.show_confirm = false;
                            if let Some(issue) = engine.selected_issue() {
                                app_state.fixing_in_progress = true;
                                app_state.fix_progress = Some(0);
                                app_state.fix_handle = Some(run_fix_with_progress(issue.fix.clone()));
                            }
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            // No - show manual commands
                            app_state.show_confirm = false;
                            app_state.show_manual_commands = true;
                        }
                        KeyCode::Esc => {
                            // Cancel - just close dialog
                            app_state.show_confirm = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => {
                        if app_state.show_detail {
                            app_state.show_detail = false;
                            app_state.clear_results();
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Esc => {
                        if app_state.show_detail {
                            app_state.show_detail = false;
                            app_state.clear_results();
                        }
                    }
                    KeyCode::Char('d') => {
                        if app_state.show_detail && !app_state.fixing_in_progress && !app_state.searching && !app_state.show_confirm {
                            if let Some(issue) = engine.selected_issue() {
                                app_state.detection_result = Some(run_detection(&issue.detection));
                            }
                        }
                    }
                    KeyCode::Char('f') => {
                        if app_state.show_detail && !app_state.fixing_in_progress && !app_state.searching && !app_state.show_confirm {
                            if let Some(_issue) = engine.selected_issue() {
                                // Only show confirm if issue was detected and no fix already applied
                                if let Some(ref detection) = app_state.detection_result {
                                    if detection.issue_found && app_state.fix_result.is_none() {
                                        app_state.show_confirm = true;
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('g') => {
                        if app_state.show_detail && !app_state.fixing_in_progress && !app_state.searching && !app_state.show_confirm {
                            if let Some(issue) = engine.selected_issue() {
                                let query = issue.symptoms.first()
                                    .map(|s| s.as_str())
                                    .unwrap_or(&issue.name);

                                app_state.searching = true;
                                terminal.draw(|f| ui::draw(f, engine, app_state))?;
                                app_state.search_result = Some(search_issues(query));
                                app_state.searching = false;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if !app_state.show_detail && !engine.issues.is_empty() {
                            app_state.show_detail = true;
                            app_state.clear_results();
                        }
                    }
                    KeyCode::Down => {
                        if !app_state.show_detail {
                            engine.move_down();
                        }
                    }
                    KeyCode::Up => {
                        if !app_state.show_detail {
                            engine.move_up();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
