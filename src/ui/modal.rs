use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui_textarea::TextArea;

use super::theme;
use crate::app::{App, InputKind, Modal};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let Some(modal) = app.modal.as_mut() else {
        return;
    };

    match modal {
        Modal::Input {
            kind,
            textarea,
            error,
        } => draw_input(frame, *kind, textarea.as_mut(), error.as_deref()),
        Modal::Help => draw_help(frame),
    }
}

fn draw_input(
    frame: &mut Frame,
    kind: InputKind,
    textarea: &mut TextArea<'_>,
    error: Option<&str>,
) {
    let area = centered(frame.area(), 64, 9);
    frame.render_widget(Clear, area);

    let border = if error.is_some() {
        theme::error()
    } else {
        theme::accent()
    };
    let block = Block::bordered().border_style(border).title(Span::styled(
        format!(" {} ", kind.title()),
        border.add_modifier(Modifier::BOLD),
    ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner);

    frame.render_widget(
        Paragraph::new(kind.label()).style(theme::muted()),
        chunks[0],
    );

    textarea.set_block(Block::bordered().border_style(border));
    frame.render_widget(&*textarea, chunks[1]);

    let error_line = error.map_or_else(
        || Line::from(""),
        |message| Line::from(Span::styled(message.to_string(), theme::error())),
    );
    frame.render_widget(Paragraph::new(error_line), chunks[2]);

    let hints = Line::from(vec![
        Span::styled("Enter", theme::accent()),
        Span::styled(" save    ", theme::muted()),
        Span::styled("Esc", theme::accent()),
        Span::styled(" cancel", theme::muted()),
    ]);
    frame.render_widget(Paragraph::new(hints), chunks[3]);
}

fn draw_help(frame: &mut Frame) {
    let area = centered(frame.area(), 66, 18);
    frame.render_widget(Clear, area);

    let block = Block::bordered()
        .border_style(theme::accent())
        .title(Span::styled(" Help ", theme::accent_bold()));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(Span::styled("Navigation", theme::accent())),
        help_line("Tab / Shift+Tab", "move focus"),
        help_line("Up / Down", "move in list"),
        help_line("Enter", "select or open"),
        help_line("Left / Right", "change version"),
        Line::from(""),
        Line::from(Span::styled("Lists", theme::accent())),
        help_line("+ Add account", "create an offline account"),
        help_line("+ New instance", "create an instance"),
        help_line("+ Add mod", "add a mod to the instance"),
        Line::from(""),
        Line::from(Span::styled("General", theme::accent())),
        help_line("?", "toggle this help"),
        help_line("q / Ctrl+C", "quit"),
        Line::from(""),
        Line::from(Span::styled("Press any key to close.", theme::muted())),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn help_line(keys: &str, action: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {keys:<18}"), theme::accent()),
        Span::styled(action.to_string(), theme::muted()),
    ])
}

fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}
