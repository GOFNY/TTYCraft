mod modal;
mod sidebar;
mod theme;
mod workspace;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph};

use crate::app::App;

const MIN_WIDTH: u16 = 72;
const MIN_HEIGHT: u16 = 20;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        draw_too_small(frame, area);
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .split(area);

    draw_header(frame, chunks[0]);
    draw_body(frame, chunks[1], app);
    draw_footer(frame, chunks[2]);
    modal::draw(frame, app);
}

pub(super) fn panel(title: impl Into<String>, focused: bool) -> Block<'static> {
    let border = if focused {
        theme::accent()
    } else {
        theme::muted()
    };
    let title_style = if focused {
        theme::accent_bold()
    } else {
        theme::muted()
    };

    Block::bordered()
        .border_style(border)
        .title(Span::styled(format!(" {} ", title.into()), title_style))
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let chunks = Layout::horizontal([Constraint::Min(0), Constraint::Length(14)]).split(area);

    let title = Line::from(vec![
        Span::styled(" TTYCraft", theme::accent_bold()),
        Span::styled("  Terminal Minecraft launcher", theme::muted()),
    ]);
    frame.render_widget(Paragraph::new(title), chunks[0]);
    frame.render_widget(
        Paragraph::new("Offline mode")
            .alignment(Alignment::Right)
            .style(theme::muted()),
        chunks[1],
    );
}

fn draw_body(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::horizontal([Constraint::Percentage(32), Constraint::Min(0)]).split(area);
    sidebar::draw(frame, chunks[0], app);
    workspace::draw(frame, chunks[1], app);
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let hints = [
        ("Tab", "focus"),
        ("Up/Down", "move"),
        ("Enter", "select"),
        ("Left/Right", "version"),
        ("?", "help"),
        ("q", "quit"),
    ];

    let mut spans = Vec::with_capacity(hints.len() * 3);
    for (index, (key, action)) in hints.iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled("   ", theme::muted()));
        }
        spans.push(Span::styled(*key, theme::accent_bold()));
        spans.push(Span::styled(format!(" {action}"), theme::muted()));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_too_small(frame: &mut Frame, area: Rect) {
    let text = Text::from(vec![
        Line::from("Terminal too small"),
        Line::from(format!(
            "Need at least {MIN_WIDTH}x{MIN_HEIGHT}, got {}x{}",
            area.width, area.height
        )),
    ]);
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), area);
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    use super::*;

    fn render_app(width: u16, height: u16, mut app: App) {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal
            .draw(|frame| draw(frame, &mut app))
            .expect("draw should not fail");
    }

    fn render(width: u16, height: u16) {
        render_app(width, height, App::new());
    }

    #[test]
    fn renders_supported_layout() {
        render(100, 30);
    }

    #[test]
    fn renders_small_terminal_fallback() {
        render(40, 10);
    }

    #[test]
    fn renders_account_modal() {
        let mut app = App::new();
        app.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        render_app(100, 30, app);
    }

    #[test]
    fn renders_help_modal() {
        let mut app = App::new();
        app.on_key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
        render_app(100, 30, app);
    }
}
