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

mod detector;
mod engine;
mod fixer;
mod github;
mod issue;
mod ui;

use detector::{run_detection, DetectionResult};
use engine::Engine;
use fixer::{run_fix, FixResult};
use github::{search_issues, SearchResult};

struct AppState {
    show_detail: bool,
    detection_result: Option<DetectionResult>,
    fix_result: Option<FixResult>,
    fixing_in_progress: bool,
    search_result: Option<SearchResult>,
    searching: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            show_detail: false,
            detection_result: None,
            fix_result: None,
            fixing_in_progress: false,
            search_result: None,
            searching: false,
        }
    }

    fn clear_results(&mut self) {
        self.detection_result = None;
        self.fix_result = None;
        self.fixing_in_progress = false;
        self.search_result = None;
        self.searching = false;
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Clear(ClearType::All))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut engine = Engine::new();
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, engine: &mut Engine, app_state: &mut AppState) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, engine, app_state))?;

        if let Event::Key(key) = event::read()? {
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
                    if app_state.show_detail && !app_state.fixing_in_progress && !app_state.searching {
                        if let Some(issue) = engine.selected_issue() {
                            // Don't clear everything - user might want to see search + detection together
                            app_state.detection_result = Some(run_detection(&issue.detection));
                        }
                    }
                }
                KeyCode::Char('f') => {
                    if app_state.show_detail && !app_state.fixing_in_progress && !app_state.searching {
                        if let Some(issue) = engine.selected_issue() {
                            // Only allow fix if issue was detected
                            if let Some(ref detection) = app_state.detection_result {
                                if detection.issue_found {
                                    app_state.fixing_in_progress = true;
                                    // Draw immediately to show "Running..." message
                                    terminal.draw(|f| ui::draw(f, engine, app_state))?;
                                    // Run the fix (blocking)
                                    app_state.fix_result = Some(run_fix(&issue.fix));
                                    app_state.fixing_in_progress = false;
                                }
                            }
                        }
                    }
                }
                KeyCode::Char('g') => {
                    if app_state.show_detail && !app_state.fixing_in_progress && !app_state.searching {
                        if let Some(issue) = engine.selected_issue() {
                            // Build query from first symptom or name
                            let query = issue.symptoms.first()
                                .map(|s| s.as_str())
                                .unwrap_or(&issue.name);

                            app_state.searching = true;
                            // Draw immediately to show "Searching..." message
                            terminal.draw(|f| ui::draw(f, engine, app_state))?;
                            // Search GitHub (blocking)
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
