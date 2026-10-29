use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use ratatui_textarea::TextArea;

use crate::accounts::{self, Account};

pub const VERSIONS: &[&str] = &[
    "1.21.8", "1.21.4", "1.20.6", "1.20.1", "1.19.4", "1.18.2", "1.16.5", "1.12.2", "1.8.9",
];

const MAX_LOG_LINES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Accounts,
    Instances,
    Mods,
    Version,
    Play,
}

impl Focus {
    pub fn next(self) -> Self {
        match self {
            Self::Accounts => Self::Instances,
            Self::Instances => Self::Mods,
            Self::Mods => Self::Version,
            Self::Version => Self::Play,
            Self::Play => Self::Accounts,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            Self::Accounts => Self::Play,
            Self::Instances => Self::Accounts,
            Self::Mods => Self::Instances,
            Self::Version => Self::Mods,
            Self::Play => Self::Version,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Account,
    Instance,
    Mod,
}

impl InputKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::Account => "Add offline account",
            Self::Instance => "New instance",
            Self::Mod => "Add mod",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Account => "Username",
            Self::Instance => "Instance name",
            Self::Mod => "Mod name",
        }
    }

    pub fn placeholder(self) -> &'static str {
        match self {
            Self::Account => "Steve",
            Self::Instance => "Fabric 1.21.8",
            Self::Mod => "Sodium",
        }
    }

    pub fn max_len(self) -> usize {
        match self {
            Self::Account => accounts::USERNAME_MAX_LEN,
            Self::Instance => 32,
            Self::Mod => 64,
        }
    }
}

pub enum Modal {
    Input {
        kind: InputKind,
        textarea: Box<TextArea<'static>>,
        error: Option<String>,
    },
    Help,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub name: String,
    pub version: usize,
    pub mods: Vec<String>,
}

pub struct App {
    pub focus: Focus,
    pub should_quit: bool,
    pub accounts: Vec<Account>,
    pub accounts_state: ListState,
    pub active_account: Option<usize>,
    pub instances: Vec<Instance>,
    pub instances_state: ListState,
    pub active_instance: Option<usize>,
    pub mods_state: ListState,
    pub modal: Option<Modal>,
    pub logs: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        let mut accounts_state = ListState::default();
        accounts_state.select(Some(0));

        let mut instances_state = ListState::default();
        instances_state.select(Some(0));

        let mut mods_state = ListState::default();
        mods_state.select(Some(0));

        Self {
            focus: Focus::Accounts,
            should_quit: false,
            // Seed data keeps the shell useful until persisted state lands
            accounts: vec![Account::offline("Steve")],
            accounts_state,
            active_account: Some(0),
            instances: vec![
                Instance {
                    name: "Vanilla".into(),
                    version: 0,
                    mods: Vec::new(),
                },
                Instance {
                    name: "Fabric".into(),
                    version: 0,
                    mods: vec!["Sodium".into(), "Iris".into()],
                },
            ],
            instances_state,
            active_instance: Some(0),
            mods_state,
            modal: None,
            logs: vec![
                "TTYCraft 0.1.0".into(),
                "UI shell ready. The launch pipeline arrives in a later change.".into(),
            ],
        }
    }

    pub fn active_account(&self) -> Option<&Account> {
        self.active_account
            .and_then(|index| self.accounts.get(index))
    }

    pub fn active_instance(&self) -> Option<&Instance> {
        self.active_instance
            .and_then(|index| self.instances.get(index))
    }

    pub fn can_play(&self) -> bool {
        self.active_account.is_some() && self.active_instance.is_some()
    }

    pub fn status(&self) -> String {
        match (self.active_account(), self.active_instance()) {
            (Some(account), Some(instance)) => format!(
                "Ready / {} / {} {}",
                account.name,
                instance.name,
                version_label(instance)
            ),
            (None, Some(_)) => "Select an account to play.".to_string(),
            (Some(_), None) => "Create an instance to play.".to_string(),
            (None, None) => "Add an account and create an instance.".to_string(),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if self.modal.is_some() {
            self.on_modal_key(key);
            return;
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.modal = Some(Modal::Help),
            KeyCode::Tab => self.focus = self.focus.next(),
            KeyCode::BackTab => self.focus = self.focus.previous(),
            KeyCode::Up => self.move_cursor(-1),
            KeyCode::Down => self.move_cursor(1),
            KeyCode::Left => self.change_version(-1),
            KeyCode::Right => self.change_version(1),
            KeyCode::Enter => self.activate(),
            _ => {}
        }
    }

    fn on_modal_key(&mut self, key: KeyEvent) {
        if matches!(self.modal, Some(Modal::Help)) {
            self.modal = None;
            return;
        }

        let action = match self.modal.as_mut() {
            Some(Modal::Input {
                kind,
                textarea,
                error,
            }) => match key.code {
                KeyCode::Esc => ModalAction::Cancel,
                KeyCode::Enter => {
                    let value = textarea.lines().join(" ").trim().to_string();
                    ModalAction::Submit(*kind, value)
                }
                _ => {
                    textarea.input(key);
                    *error = None;
                    ModalAction::None
                }
            },
            _ => ModalAction::None,
        };

        match action {
            ModalAction::Cancel => self.modal = None,
            ModalAction::Submit(kind, value) => match self.submit_input(kind, &value) {
                Ok(()) => self.modal = None,
                Err(message) => {
                    if let Some(Modal::Input { error, .. }) = self.modal.as_mut() {
                        *error = Some(message.to_string());
                    }
                }
            },
            ModalAction::None => {}
        }
    }

    fn submit_input(&mut self, kind: InputKind, value: &str) -> Result<(), &'static str> {
        if value.is_empty() {
            return Err("Name cannot be empty.");
        }
        if value.chars().count() > kind.max_len() {
            return Err("Name is too long.");
        }

        match kind {
            InputKind::Account => {
                accounts::validate_username(value)?;
                if self
                    .accounts
                    .iter()
                    .any(|account| account.name.eq_ignore_ascii_case(value))
                {
                    return Err("That account already exists.");
                }

                self.accounts.push(Account::offline(value));
                let index = self.accounts.len() - 1;
                self.active_account = Some(index);
                self.accounts_state.select(Some(index));
                self.push_log(format!("Added offline account '{value}'."));
            }
            InputKind::Instance => {
                if self
                    .instances
                    .iter()
                    .any(|instance| instance.name.eq_ignore_ascii_case(value))
                {
                    return Err("An instance with that name already exists.");
                }

                self.instances.push(Instance {
                    name: value.to_string(),
                    version: 0,
                    mods: Vec::new(),
                });
                let index = self.instances.len() - 1;
                self.active_instance = Some(index);
                self.instances_state.select(Some(index));
                self.mods_state.select(Some(0));
                self.push_log(format!("Created instance '{value}'."));
            }
            InputKind::Mod => {
                let Some(index) = self.active_instance else {
                    return Err("Select an instance first.");
                };
                let Some(instance) = self.instances.get_mut(index) else {
                    return Err("Select an instance first.");
                };

                if instance
                    .mods
                    .iter()
                    .any(|installed| installed.eq_ignore_ascii_case(value))
                {
                    return Err("That mod is already in this instance.");
                }

                instance.mods.push(value.to_string());
                let mod_index = instance.mods.len() - 1;
                let instance_name = instance.name.clone();
                self.mods_state.select(Some(mod_index));
                self.push_log(format!("Added mod '{value}' to '{instance_name}'."));
            }
        }

        Ok(())
    }

    fn open_input(&mut self, kind: InputKind) {
        let mut textarea = TextArea::default();
        textarea.set_tab_length(0);
        textarea.set_placeholder_text(kind.placeholder());
        self.modal = Some(Modal::Input {
            kind,
            textarea: Box::new(textarea),
            error: None,
        });
    }

    fn move_cursor(&mut self, delta: isize) {
        match self.focus {
            Focus::Accounts => {
                move_list(&mut self.accounts_state, self.accounts.len() + 1, delta);
            }
            Focus::Instances => {
                move_list(&mut self.instances_state, self.instances.len() + 1, delta);
            }
            Focus::Mods => {
                let rows = self
                    .active_instance()
                    .map_or(0, |instance| instance.mods.len() + 1);
                move_list(&mut self.mods_state, rows, delta);
            }
            Focus::Version | Focus::Play => {}
        }
    }

    fn change_version(&mut self, delta: isize) {
        if self.focus != Focus::Version {
            return;
        }

        let Some(index) = self.active_instance else {
            return;
        };
        let Some(instance) = self.instances.get_mut(index) else {
            return;
        };

        let last = VERSIONS.len().saturating_sub(1);
        instance.version = if delta < 0 {
            instance.version.saturating_sub(delta.unsigned_abs())
        } else {
            instance.version.saturating_add(delta as usize).min(last)
        };
    }

    fn activate(&mut self) {
        match self.focus {
            Focus::Accounts => {
                let cursor = self.accounts_state.selected().unwrap_or(0);
                if cursor < self.accounts.len() {
                    self.active_account = Some(cursor);
                    let name = self.accounts[cursor].name.clone();
                    self.push_log(format!("Account '{name}' selected."));
                } else {
                    self.open_input(InputKind::Account);
                }
            }
            Focus::Instances => {
                let cursor = self.instances_state.selected().unwrap_or(0);
                if cursor < self.instances.len() {
                    self.active_instance = Some(cursor);
                    self.mods_state.select(Some(0));
                    let name = self.instances[cursor].name.clone();
                    self.push_log(format!("Instance '{name}' selected."));
                } else {
                    self.open_input(InputKind::Instance);
                }
            }
            Focus::Mods => {
                let Some(instance) = self.active_instance() else {
                    return;
                };
                let cursor = self.mods_state.selected().unwrap_or(0);
                if cursor < instance.mods.len() {
                    let name = instance.mods[cursor].clone();
                    let instance_name = instance.name.clone();
                    self.push_log(format!("Mod '{name}' selected in '{instance_name}'."));
                } else {
                    self.open_input(InputKind::Mod);
                }
            }
            Focus::Version => {}
            Focus::Play => self.play(),
        }
    }

    fn play(&mut self) {
        let Some(account) = self.active_account() else {
            self.push_log("Select an account before playing.");
            return;
        };
        let Some(instance) = self.active_instance() else {
            self.push_log("Create an instance before playing.");
            return;
        };

        let message = format!(
            "Launch requested: {} -> {} ({}). The pipeline is not wired yet.",
            account.name,
            instance.name,
            version_label(instance)
        );
        self.push_log(message);
    }

    fn push_log(&mut self, message: impl Into<String>) {
        if self.logs.len() >= MAX_LOG_LINES {
            self.logs.remove(0);
        }
        self.logs.push(message.into());
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub fn version_label(instance: &Instance) -> &'static str {
    VERSIONS.get(instance.version).copied().unwrap_or("unknown")
}

fn move_list(state: &mut ListState, rows: usize, delta: isize) {
    if rows == 0 {
        state.select(None);
        return;
    }

    let current = state.selected().unwrap_or(0);
    let next = if delta < 0 {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        current.saturating_add(delta as usize).min(rows - 1)
    };
    state.select(Some(next));
}

enum ModalAction {
    None,
    Cancel,
    Submit(InputKind, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_cycles_in_both_directions() {
        assert_eq!(Focus::Accounts.previous(), Focus::Play);
        assert_eq!(Focus::Play.next(), Focus::Accounts);
    }

    #[test]
    fn version_selection_stays_within_bounds() {
        let mut app = App::new();
        app.focus = Focus::Version;

        for _ in 0..VERSIONS.len() + 5 {
            app.change_version(-1);
        }
        assert_eq!(app.instances[0].version, 0);

        for _ in 0..VERSIONS.len() + 5 {
            app.change_version(1);
        }
        assert_eq!(app.instances[0].version, VERSIONS.len() - 1);
    }

    #[test]
    fn rejects_duplicate_accounts_case_insensitively() {
        let mut app = App::new();
        let error = app.submit_input(InputKind::Account, "steve").unwrap_err();
        assert_eq!(error, "That account already exists.");
    }

    #[test]
    fn adds_account_and_selects_it() {
        let mut app = App::new();
        app.submit_input(InputKind::Account, "Alex").unwrap();
        assert_eq!(
            app.accounts.last().map(|account| account.name.as_str()),
            Some("Alex")
        );
        assert_eq!(app.active_account, Some(app.accounts.len() - 1));
    }

    #[test]
    fn account_modal_submits_typed_name() {
        let mut app = App::new();

        app.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert!(matches!(app.modal, Some(Modal::Input { .. })));

        for character in "Alex".chars() {
            app.on_key(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
        }
        app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));

        assert!(app.accounts.iter().any(|account| account.name == "Alex"));
        assert!(app.modal.is_none());
    }
}
