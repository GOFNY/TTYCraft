use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, Paragraph};

use super::{panel, theme};
use crate::app::{App, Focus, Instance, version_label};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::vertical([
        Constraint::Length(list_height(app.accounts.len(), 10)),
        Constraint::Length(list_height(app.instances.len(), 10)),
        Constraint::Min(4),
    ])
    .split(area);

    draw_accounts(frame, chunks[0], app);
    draw_instances(frame, chunks[1], app);
    draw_mods(frame, chunks[2], app);
}

fn list_height(items: usize, max: u16) -> u16 {
    (items as u16).saturating_add(3).min(max)
}

fn draw_accounts(frame: &mut Frame, area: Rect, app: &mut App) {
    let focused = app.focus == Focus::Accounts;
    let items = account_items(app);
    let list = List::new(items)
        .block(panel("Accounts", focused))
        .highlight_style(if focused {
            theme::selected()
        } else {
            Style::default()
        });

    frame.render_stateful_widget(list, area, &mut app.accounts_state);
}

fn account_items(app: &App) -> Vec<ListItem<'static>> {
    let mut items = Vec::with_capacity(app.accounts.len() + 1);

    for (index, account) in app.accounts.iter().enumerate() {
        let active = app.active_account == Some(index);
        let marker = if active { "●" } else { " " };
        let style = if active {
            theme::active()
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {marker} "), style),
            Span::styled(account.name.clone(), style),
            Span::styled("  Offline", theme::muted()),
        ])));
    }

    items.push(ListItem::new(Line::from(Span::styled(
        " + Add account",
        theme::accent(),
    ))));

    items
}

fn draw_instances(frame: &mut Frame, area: Rect, app: &mut App) {
    let focused = app.focus == Focus::Instances;
    let items = instance_items(app);
    let list = List::new(items)
        .block(panel("Instances", focused))
        .highlight_style(if focused {
            theme::selected()
        } else {
            Style::default()
        });

    frame.render_stateful_widget(list, area, &mut app.instances_state);
}

fn instance_items(app: &App) -> Vec<ListItem<'static>> {
    let mut items = Vec::with_capacity(app.instances.len() + 1);

    for (index, instance) in app.instances.iter().enumerate() {
        let active = app.active_instance == Some(index);
        let marker = if active { "●" } else { " " };
        let style = if active {
            theme::active()
        } else {
            Style::default()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {marker} "), style),
            Span::styled(instance.name.clone(), style),
            Span::styled(format!("  {}", version_label(instance)), theme::muted()),
        ])));
    }

    items.push(ListItem::new(Line::from(Span::styled(
        " + New instance",
        theme::accent(),
    ))));

    items
}

fn draw_mods(frame: &mut Frame, area: Rect, app: &mut App) {
    let focused = app.focus == Focus::Mods;

    let Some(instance) = app.active_instance() else {
        app.mods_state.select(None);
        let block = panel("Mods", focused);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("Select an instance first.").style(theme::muted()),
            inner,
        );
        return;
    };

    let title = format!("Mods ({})", instance.mods.len());
    let items = mod_items(instance);
    let list = List::new(items)
        .block(panel(title, focused))
        .highlight_style(if focused {
            theme::selected()
        } else {
            Style::default()
        });

    frame.render_stateful_widget(list, area, &mut app.mods_state);
}

fn mod_items(instance: &Instance) -> Vec<ListItem<'static>> {
    let mut items = Vec::with_capacity(instance.mods.len() + 1);

    for name in &instance.mods {
        items.push(ListItem::new(Line::from(vec![
            Span::raw("   "),
            Span::raw(name.clone()),
        ])));
    }

    items.push(ListItem::new(Line::from(Span::styled(
        " + Add mod",
        theme::accent(),
    ))));

    items
}
