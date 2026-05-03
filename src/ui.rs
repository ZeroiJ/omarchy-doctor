use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::engine::Engine;
use crate::github::GitHubIssue;
use crate::issue::Issue;
use crate::AppState;

pub fn draw(frame: &mut Frame, engine: &Engine, app_state: &AppState) {
    if app_state.show_detail {
        draw_detail(frame, engine, app_state);
    } else {
        draw_list(frame, engine);
    }
}

fn draw_list(frame: &mut Frame, engine: &Engine) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.size());

    let main_block = Block::default()
        .title("🔧 OMARCHY DOCTOR v1.0")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = main_block.inner(chunks[0]);
    frame.render_widget(main_block, chunks[0]);

    if engine.issues.is_empty() {
        let empty_msg = Paragraph::new("No issues found. Add TOML files to the fixes/ directory.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(empty_msg, inner_area);
    } else {
        let items: Vec<ListItem> = engine
            .issues
            .iter()
            .enumerate()
            .map(|(i, issue)| {
                let style = if i == engine.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(issue.display_name()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::NONE))
            .highlight_symbol("");

        frame.render_widget(list, inner_area);
    }

    let help_text = Line::from(vec![
        Span::styled("[↑↓] Navigate  ", Style::default().fg(Color::Gray)),
        Span::styled("[Enter] Select  ", Style::default().fg(Color::Gray)),
        Span::styled("[q] Quit", Style::default().fg(Color::Gray)),
    ]);

    let help_bar = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(help_bar, chunks[1]);
}

fn draw_detail(frame: &mut Frame, engine: &Engine, app_state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.size());

    let main_block = Block::default()
        .title("🔧 OMARCHY DOCTOR v1.0")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner_area = main_block.inner(chunks[0]);
    frame.render_widget(main_block, chunks[0]);

    if let Some(issue) = engine.selected_issue() {
        render_issue_detail(frame, inner_area, issue, app_state);
    }

    let help_spans = build_help_bar(app_state);
    let help_text = Line::from(help_spans);
    let help_bar = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(help_bar, chunks[1]);
}

fn build_help_bar(app_state: &AppState) -> Vec<Span<'_>> {
    let mut spans = vec![];

    // Always show [g] Search GitHub on detail screen
    if !app_state.searching {
        spans.push(Span::styled("[g] Search GitHub  ", Style::default().fg(Color::Gray)));
    }

    // Show [f] Fix if issue detected and not already fixing or fixed
    if let Some(ref detection) = app_state.detection_result {
        if detection.issue_found {
            if app_state.fixing_in_progress {
                // No fix option while fixing
            } else if app_state.fix_result.is_none() {
                spans.push(Span::styled("[f] Fix  ", Style::default().fg(Color::Gray)));
            }
        }
    }

    // Show detect/re-detect option
    if app_state.detection_result.is_some() || app_state.fix_result.is_some() {
        spans.push(Span::styled("[d] Re-detect  ", Style::default().fg(Color::Gray)));
    } else {
        spans.push(Span::styled("[d] Detect  ", Style::default().fg(Color::Gray)));
    }

    spans.push(Span::styled("[Esc] Back  ", Style::default().fg(Color::Gray)));
    spans.push(Span::styled("[q] Quit", Style::default().fg(Color::Gray)));

    spans
}

fn render_issue_detail(
    frame: &mut Frame,
    area: Rect,
    issue: &Issue,
    app_state: &AppState,
) {
    // Calculate constraints based on what needs to be shown
    let mut constraints = vec![
        Constraint::Length(2),  // Name
        Constraint::Length(2),  // Category
        Constraint::Length(2),  // Symptoms
        Constraint::Length(3),  // Detection
        Constraint::Length(3),  // Fix
        Constraint::Length(2),  // Safety
    ];

    // Add space for detection result if present
    if app_state.detection_result.is_some() {
        constraints.push(Constraint::Length(4));
    }

    // Add space for fixing in progress message
    if app_state.fixing_in_progress {
        constraints.push(Constraint::Length(3));
    }

    // Add space for fix result if present
    if app_state.fix_result.is_some() {
        constraints.push(Constraint::Length(4));
    }

    // Add space for search in progress message
    if app_state.searching {
        constraints.push(Constraint::Length(3));
    }

    // Add space for search results if present
    if app_state.search_result.is_some() {
        constraints.push(Constraint::Length(10));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut chunk_idx = 0;

    // Name (large, bold)
    let name = Paragraph::new(issue.display_name())
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left);
    frame.render_widget(name, chunks[chunk_idx]);
    chunk_idx += 1;

    // Category
    let category_text = Line::from(vec![
        Span::styled("Category: ", Style::default().fg(Color::Gray)),
        Span::styled(&issue.category, Style::default().fg(Color::Cyan)),
    ]);
    let category = Paragraph::new(category_text);
    frame.render_widget(category, chunks[chunk_idx]);
    chunk_idx += 1;

    // Symptoms (comma-separated)
    let symptoms_str = issue.symptoms.join(", ");
    let symptoms_text = Line::from(vec![
        Span::styled("Symptoms: ", Style::default().fg(Color::Gray)),
        Span::styled(symptoms_str, Style::default().fg(Color::White)),
    ]);
    let symptoms = Paragraph::new(symptoms_text);
    frame.render_widget(symptoms, chunks[chunk_idx]);
    chunk_idx += 1;

    // Detection command box (yellow)
    let detection_block = Block::default()
        .title("Detection Command")
        .title_style(Style::default().fg(Color::Yellow))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let detection = Paragraph::new(&issue.detection as &str)
        .block(detection_block)
        .style(Style::default().fg(Color::Yellow))
        .wrap(Wrap { trim: false });
    frame.render_widget(detection, chunks[chunk_idx]);
    chunk_idx += 1;

    // Fix command box (green)
    let fix_block = Block::default()
        .title("Fix Command")
        .title_style(Style::default().fg(Color::Green))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let fix = Paragraph::new(&issue.fix as &str)
        .block(fix_block)
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false });
    frame.render_widget(fix, chunks[chunk_idx]);
    chunk_idx += 1;

    // Safe to auto-run
    let safety_color = if issue.safe { Color::Green } else { Color::Red };
    let safety_text = if issue.safe { "✓ Safe to auto-run" } else { "✗ Manual intervention required" };
    let safety = Paragraph::new(safety_text)
        .style(Style::default().fg(safety_color).add_modifier(Modifier::BOLD));
    frame.render_widget(safety, chunks[chunk_idx]);
    chunk_idx += 1;

    // Detection result (if available)
    if let Some(ref result) = app_state.detection_result {
        if chunk_idx < chunks.len() {
            if result.issue_found {
                // Red box - issue detected
                let result_block = Block::default()
                    .title("⚠️ ISSUE DETECTED — needs fixing")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .style(Style::default().bg(Color::Red));

                let result_text = if result.output.is_empty() {
                    "The detection command indicated this issue is present on your system.".to_string()
                } else {
                    format!("Output: {}", result.output)
                };

                let result_para = Paragraph::new(result_text)
                    .block(result_block)
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: false });
                frame.render_widget(result_para, chunks[chunk_idx]);
            } else {
                // Green box - no issue
                let result_block = Block::default()
                    .title("✅ No issue found")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .style(Style::default().bg(Color::Green));

                let result_text = if result.output.is_empty() {
                    "The detection command indicated no issue on your system.".to_string()
                } else {
                    format!("Output: {}", result.output)
                };

                let result_para = Paragraph::new(result_text)
                    .block(result_block)
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: false });
                frame.render_widget(result_para, chunks[chunk_idx]);
            }
            chunk_idx += 1;
        }
    }

    // Fix in progress message
    if app_state.fixing_in_progress {
        if chunk_idx < chunks.len() {
            let progress_block = Block::default()
                .title("🔄 Running fix...")
                .title_style(Style::default().fg(Color::Black))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Yellow));

            let progress = Paragraph::new("Please wait while the fix command executes...")
                .block(progress_block)
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center);
            frame.render_widget(progress, chunks[chunk_idx]);
            chunk_idx += 1;
        }
    }

    // Fix result (if available)
    if let Some(ref result) = app_state.fix_result {
        if chunk_idx < chunks.len() {
            if result.success {
                // Green box - fix succeeded
                let result_block = Block::default()
                    .title("✅ Fix applied successfully!")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .style(Style::default().bg(Color::Green));

                let result_text = if result.output.is_empty() {
                    "The fix command completed successfully.".to_string()
                } else {
                    format!("Output:\n{}", result.output)
                };

                let result_para = Paragraph::new(result_text)
                    .block(result_block)
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: false });
                frame.render_widget(result_para, chunks[chunk_idx]);
            } else {
                // Red box - fix failed
                let result_block = Block::default()
                    .title("❌ Fix failed")
                    .title_style(Style::default().fg(Color::White))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red))
                    .style(Style::default().bg(Color::Red));

                let result_text = if result.output.is_empty() {
                    "The fix command returned an error.".to_string()
                } else {
                    format!("Output:\n{}", result.output)
                };

                let result_para = Paragraph::new(result_text)
                    .block(result_block)
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: false });
                frame.render_widget(result_para, chunks[chunk_idx]);
            }
            chunk_idx += 1;
        }
    }

    // Search in progress message
    if app_state.searching {
        if chunk_idx < chunks.len() {
            let search_block = Block::default()
                .title("🔍 Searching GitHub...")
                .title_style(Style::default().fg(Color::Black))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Cyan));

            let search_msg = Paragraph::new("Searching basecamp/omarchy repository...")
                .block(search_block)
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center);
            frame.render_widget(search_msg, chunks[chunk_idx]);
            chunk_idx += 1;
        }
    }

    // Search results (if available)
    if let Some(ref result) = app_state.search_result {
        if chunk_idx < chunks.len() {
            if result.success {
                render_github_results(frame, chunks[chunk_idx], &result.issues);
            } else {
                // Error box
                let error_block = Block::default()
                    .title("⚠️ GitHub Search Error")
                    .title_style(Style::default().fg(Color::Black))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Yellow));

                let error_msg = Paragraph::new(&result.error as &str)
                    .block(error_block)
                    .style(Style::default().fg(Color::Black))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: false });
                frame.render_widget(error_msg, chunks[chunk_idx]);
            }
        }
    }
}

fn render_github_results(frame: &mut Frame, area: Rect, issues: &[GitHubIssue]) {
    let mut text_lines: Vec<Line> = vec![];

    if issues.is_empty() {
        text_lines.push(Line::from(vec![
            Span::styled("No related issues found on GitHub.", Style::default().fg(Color::Gray)),
        ]));
    } else {
        // Title
        text_lines.push(Line::from(vec![
            Span::styled(
                format!("Found {} related issue(s) on basecamp/omarchy:", issues.len().min(3)),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]));
        text_lines.push(Line::from(""));

        // Show up to 3 issues
        for (i, issue) in issues.iter().take(3).enumerate() {
            let status_icon = if issue.state == "open" {
                "🟢 Open"
            } else {
                "🔴 Closed"
            };
            let status_color = if issue.state == "open" {
                Color::Green
            } else {
                Color::Red
            };

            // Issue number and status
            text_lines.push(Line::from(vec![
                Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Gray)),
                Span::styled(status_icon, Style::default().fg(status_color)),
            ]));

            // Title
            text_lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(&issue.title, Style::default().fg(Color::White)),
            ]));

            // URL (dimmed)
            text_lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(&issue.html_url, Style::default().fg(Color::DarkGray)),
            ]));

            if i < issues.len().min(3) - 1 {
                text_lines.push(Line::from(""));
            }
        }
    }

    let block = Block::default()
        .title("GitHub Issues (basecamp/omarchy)")
        .title_style(Style::default().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let results = Paragraph::new(text_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(results, area);
}
