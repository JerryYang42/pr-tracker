use crate::model::{FetchResult, PullRequest};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::time::Duration;
use tracing::debug;

/// Application state.
pub struct App<'a> {
    pub prs: Vec<&'a PullRequest>,
    pub table_state: TableState,
    pub warnings: Vec<String>,
    pub status_msg: Option<String>,
}

impl<'a> App<'a> {
    pub fn new(prs: Vec<&'a PullRequest>, results: &[FetchResult]) -> Self {
        let warnings = results.iter().filter_map(|r| r.warning.clone()).collect();

        let mut table_state = TableState::default();
        if !prs.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            prs,
            table_state,
            warnings,
            status_msg: None,
        }
    }

    pub fn selected_pr(&self) -> Option<&PullRequest> {
        self.table_state
            .selected()
            .and_then(|i| self.prs.get(i).copied())
    }

    pub fn move_down(&mut self) {
        let next = match self.table_state.selected() {
            Some(i) if i + 1 < self.prs.len() => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.table_state.select(Some(next));
    }

    pub fn move_up(&mut self) {
        let prev = match self.table_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => 0,
        };
        self.table_state.select(Some(prev));
    }
}

/// Run the interactive TUI event loop.
pub fn run(mut terminal: DefaultTerminal, results: Vec<FetchResult>) -> Result<()> {
    let sorted = crate::fetch::sorted_prs(&results);
    let mut app = App::new(sorted, &results);

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                // Only act on key-press events, not releases.
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,

                    KeyCode::Char('j') | KeyCode::Down => {
                        app.move_down();
                        app.status_msg = None;
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.move_up();
                        app.status_msg = None;
                    }

                    KeyCode::Char('o') | KeyCode::Enter => {
                        if let Some(pr) = app.selected_pr() {
                            debug!("opening {}", pr.url);
                            match open::that(&pr.url) {
                                Ok(()) => {
                                    app.status_msg =
                                        Some(format!("Opened PR #{} in browser", pr.number));
                                }
                                Err(e) => {
                                    app.status_msg = Some(format!("Could not open browser: {e}"));
                                }
                            }
                        }
                    }

                    // 'r' is handled by the outer caller which re-fetches and re-runs the TUI.
                    // Signal by returning Ok — the caller decides whether to re-enter.
                    KeyCode::Char('r') => {
                        app.status_msg = Some("Refreshing…".into());
                        terminal.draw(|f| draw(f, &mut app))?;
                        break;
                    }

                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    // Layout: header bar | table | footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title bar
            Constraint::Min(5),    // table
            Constraint::Length(3), // footer / status
        ])
        .split(area);

    // ── Title bar ────────────────────────────────────────────────────────────
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " pr-tracker ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  {} open PR(s)", app.prs.len())),
    ]));
    frame.render_widget(title, chunks[0]);

    // ── PR table ─────────────────────────────────────────────────────────────
    let header_cells = ["#", "Repository", "Title", "Author", "Updated"]
        .iter()
        .map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let rows = app.prs.iter().map(|pr| {
        let updated = pr.updated_at.format("%Y-%m-%d %H:%M").to_string();
        let draft_marker = if pr.draft { " [draft]" } else { "" };
        Row::new(vec![
            Cell::from(format!("#{}", pr.number)),
            Cell::from(pr.repo.clone()),
            Cell::from(format!("{}{}", pr.title, draft_marker)),
            Cell::from(pr.author.clone()),
            Cell::from(updated),
        ])
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(22),
            Constraint::Min(30),
            Constraint::Length(16),
            Constraint::Length(17),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Pull Requests"),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("► ");

    frame.render_stateful_widget(table, chunks[1], &mut app.table_state);

    // ── Footer / status bar ──────────────────────────────────────────────────
    let status_text = if let Some(msg) = &app.status_msg {
        msg.clone()
    } else if !app.warnings.is_empty() {
        format!("⚠  {}", app.warnings.join(" | "))
    } else {
        String::new()
    };

    let hint = "  j/k navigate  o open in browser  r refresh  q quit";

    let footer = Paragraph::new(vec![
        Line::from(Span::styled(
            status_text,
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(hint, Style::default().fg(Color::DarkGray))),
    ])
    .block(Block::default().borders(Borders::TOP));

    frame.render_widget(footer, chunks[2]);
}
