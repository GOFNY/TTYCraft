use ratatui::style::{Color, Modifier, Style};

pub const ACCENT: Color = Color::Rgb(0x7c, 0xbd, 0x4a);
pub const ACTIVE: Color = Color::Rgb(0xa5, 0xd9, 0x6a);
pub const MUTED: Color = Color::DarkGray;

pub fn accent() -> Style {
    Style::new().fg(ACCENT)
}

pub fn accent_bold() -> Style {
    accent().add_modifier(Modifier::BOLD)
}

pub fn muted() -> Style {
    Style::new().fg(MUTED)
}

pub fn active() -> Style {
    Style::new().fg(ACTIVE).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::new()
        .bg(Color::DarkGray)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn error() -> Style {
    Style::new().fg(Color::Red)
}
