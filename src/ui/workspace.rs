use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::{panel, theme};
use crate::app::{App, Focus, version_label};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::vertical([Constraint::Length(9), Constraint::Min(6)]).split(area);
    draw_play(frame, chunks[0], app);
    draw_console(frame, chunks[1], app);
}

fn draw_play(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.focus == Focus::Play;
    let block = panel("Play", focused);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let version = app.active_instance().map_or("--", version_label);
    let selector_style = if app.focus == Focus::Version {
        theme::accent().add_modifier(Modifier::BOLD)
    } else {
        theme::muted()
    };
    let button_style = if app.can_play() {
        Style::new()
            .bg(theme::ACCENT)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::new().bg(theme::MUTED).fg(Color::Gray)
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("<  ", selector_style),
            Span::styled(
                version,
                Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  >", selector_style),
        ]),
        Line::from(""),
        Line::from(Span::styled("  PLAY  ", button_style)),
        Line::from(""),
        Line::from(Span::styled(app.status(), theme::muted())),
    ];

    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Center), inner);
}

fn draw_console(frame: &mut Frame, area: Rect, app: &App) {
    let block = panel("Console", false);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible = inner.height as usize;
    let start = app.logs.len().saturating_sub(visible);
    let lines: Vec<Line> = app.logs[start..]
        .iter()
        .map(|message| {
            Line::from(vec![
                Span::styled("> ", theme::muted()),
                Span::raw(message.clone()),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}
