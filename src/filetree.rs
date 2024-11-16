use ratatui::{
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph},
    Frame,
};

use log::{error, info, warn, Metadata};
use std::{
    borrow::{BorrowMut, Cow},
    fs::File,
};
use std::{
    collections::btree_map::Entry,
    fs::{self, DirEntry},
    ops::Index,
};
use std::{fmt::Debug, path};
use std::{fmt::Error, os::unix::fs::PermissionsExt};
use std::{path::PathBuf, result};

use crate::app::{App, AppInfo, AppMessage};

use crossterm::event;
use crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct FileTreeApp {
    info: AppInfo,
    text: std::string::String,
    input: String,
    new_input: String,
    character_index: usize,
    open_path: String,
    entries: Vec<path::PathBuf>,
    metadata: Vec<fs::Metadata>,
    input_mode: InputMode,
    select_state: ListState,
    confirm_action: ConfirmAction,
}

pub enum FileTreeMsg {
    TextEntered(char),
    OpenPath,
    CursorLeft,
    CursorRight,
    CursorDown,
    CursorUp,
    DeleteChar,
    CreateFile,
    DeleteFile,
    Confirm,
    Cancel,
    NoneMsg,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Delete(usize),
    Rename(usize),
    None,
}

#[derive(Debug, Clone)]
enum InputMode {
    Search,
    Modify,
}

impl AppMessage for FileTreeMsg {}

impl App for FileTreeApp {
    type Msg = FileTreeMsg;
    fn view(&mut self, layout: &Layout, frame: &mut Frame, style: Style) {
        let app_area = layout.split(frame.area())[1];

        let vertical = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]);

        let [input_area, path_area] = vertical.areas(app_area);

        let mut input_style = style;
        let mut path_style = style;

        match self.input_mode {
            InputMode::Modify => {
                input_style = Style::default().fg(Color::White);
            }
            InputMode::Search => {
                path_style = Style::default().fg(Color::White);
            }
        }

        let input = Paragraph::new(self.input.as_str())
            .style(input_style)
            .block(Block::bordered().title("Enter path"));

        frame.render_widget(input, input_area);

        #[allow(clippy::cast_possible_truncation)]
        frame.set_cursor_position(Position::new(
            input_area.x + self.character_index as u16 + 1,
            input_area.y + 1,
        ));

        let mut file_items: Vec<ListItem> = Vec::new();

        for i in 0..self.entries.len() {
            let formatted_info =
                self.format_dir_data(i, self.entries[i].clone(), self.metadata[i].clone());
            let content = Line::from(Span::raw(Cow::Owned(formatted_info)));
            let item = ListItem::new(content);
            file_items.push(item)
        }

        if let InputMode::Modify = self.input_mode {
            match self.confirm_action {
                ConfirmAction::Delete(index) => {
                    let delete_path = self.entries[index].clone();
                    let confirm_delete =
                        format!("Are you sure you want to delete {:?}? (Y/N)", delete_path);
                    file_items[index] = ListItem::new(confirm_delete);
                }
                _ => {}
            }
        }
        let last_item = ListItem::new(format!("{:>3}: ", self.entries.len()));
        file_items.push(last_item);

        let list = List::new(file_items)
            .block(Block::bordered().title("Directory Contents (Path|Type|Perm|Size):"))
            .style(path_style)
            .highlight_style(Style::default().bg(Color::LightGreen));
        frame.render_stateful_widget(list, path_area, &mut self.select_state);
    }

    fn update(&mut self, msg: &Self::Msg) {
        match msg {
            FileTreeMsg::OpenPath => match self.input_mode {
                InputMode::Search => self.input_mode = InputMode::Modify,
                InputMode::Modify => self.input_mode = InputMode::Search,
            },
            FileTreeMsg::TextEntered(to_insert) => match self.input_mode {
                InputMode::Search => {
                    self.enter_char(to_insert.clone());
                    self.read_path(self.input.clone());
                }
                InputMode::Modify => {}
            },
            FileTreeMsg::CursorLeft => self.move_cursor_left(),
            FileTreeMsg::CursorRight => self.move_cursor_right(),
            FileTreeMsg::CursorDown => self.select_state.select_next(),
            FileTreeMsg::CursorUp => self.select_state.select_previous(),
            FileTreeMsg::DeleteChar => self.delete_char(),
            FileTreeMsg::CreateFile => {
                let filepath = format!("{}", self.input);
                let result = fs::File::create(&filepath);

                match result {
                    Ok(file) => {
                        info!("Created file {:?}", file);
                        self.read_path(self.open_path.clone());
                    }
                    Err(err) => error!("Could not create file: {err} at {filepath}"),
                }
            }
            FileTreeMsg::DeleteFile => {
                let index = self.select_state.selected().unwrap();
                self.confirm_action = ConfirmAction::Delete(index)
            }
            FileTreeMsg::Confirm => {
                let action = self.confirm_action.clone();
                let result = self.confirm_action();

                match result {
                    Ok(()) => {
                        info!("Confirmed action {:?}!", action)
                    }
                    Err(err) => {
                        error!("Could not confirm action {:?}! Error: {}", action, err)
                    }
                }
            }
            FileTreeMsg::Cancel => self.cancel_action(),
            _ => {}
        }
    }

    fn info(&self) -> AppInfo {
        return self.info.clone();
    }

    fn generate_msg(&self, key_event: event::KeyEvent) -> Option<Self::Msg> {
        match self.input_mode {
            InputMode::Search => match key_event.code {
                KeyCode::Enter => Some(FileTreeMsg::OpenPath),
                KeyCode::Char(' ') => Some(FileTreeMsg::CreateFile),
                KeyCode::Char(to_insert) => Some(FileTreeMsg::TextEntered(to_insert)),
                KeyCode::Backspace => Some(FileTreeMsg::DeleteChar),
                KeyCode::Left => Some(FileTreeMsg::CursorLeft),
                KeyCode::Right => Some(FileTreeMsg::CursorRight),
                _ => Some(FileTreeMsg::NoneMsg),
            },
            InputMode::Modify => match key_event.code {
                KeyCode::Backspace => Some(FileTreeMsg::OpenPath),
                KeyCode::Char('j') => Some(FileTreeMsg::CursorDown),
                KeyCode::Char('k') => Some(FileTreeMsg::CursorUp),
                KeyCode::Char('d') => Some(FileTreeMsg::DeleteFile),
                KeyCode::Char('y') | KeyCode::Char('Y') => Some(FileTreeMsg::Confirm),
                KeyCode::Char('n') | KeyCode::Char('N') => Some(FileTreeMsg::Cancel),
                _ => Some(FileTreeMsg::NoneMsg),
            },
        }
    }
}

impl FileTreeApp {
    pub fn new() -> FileTreeApp {
        let main_app = FileTreeApp {
            info: AppInfo {
                title: "File Tree".to_string(),
                version: "v1.0".to_string(),
            },
            text: "File Tree App".to_string(),
            input: String::new(),
            open_path: "/".to_string(),
            new_input: "".to_string(),
            character_index: 0,
            entries: vec![],
            metadata: vec![],
            input_mode: InputMode::Search,
            select_state: ListState::default(),
            confirm_action: ConfirmAction::None,
        };
        return main_app;
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }

        self.read_path(self.input.clone());
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn update_path(&mut self) {
        self.open_path = self.input.clone();
    }

    fn new_char(&mut self) {
        self.open_path = self.input.clone();
    }

    fn format_dir_data(&self, num: usize, path: PathBuf, metadata: fs::Metadata) -> String {
        let mut icon = "";
        let perms = metadata.permissions();
        let mode = perms.mode();
        let size = metadata.len() / 1024;

        if metadata.is_file() {
            icon = "\u{f15c}";
        } else if metadata.is_dir() {
            icon = "\u{e5fe}";
        } else if metadata.is_symlink() {
            icon = "\u{f0337}";
        }

        let path_string = path.display();

        let format_triplet = |bits: u32| {
            format!(
                "{}{}{}",
                if bits & 0b100 != 0 { 'r' } else { '-' },
                if bits & 0b010 != 0 { 'w' } else { '-' },
                if bits & 0b001 != 0 { 'x' } else { '-' },
            )
        };

        let perm_str = format!(
            "{}{}{}",
            format_triplet((mode >> 6) & 0b111),
            format_triplet((mode >> 3) & 0b111),
            format_triplet(mode & 0b111),
        );

        format!("{num:>3}: {path_string:<60} | {icon} | {perm_str:<10} | {size:>4}KB",)
    }

    fn read_path(&mut self, path: String) {
        let result = std::fs::read_dir(&path);

        match result {
            Ok(dir_content) => {
                self.open_path = path.clone();
                self.entries.clear();

                let (paths, metadatas): (Vec<PathBuf>, Vec<fs::Metadata>) = dir_content
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|e| Some((e.path(), e.metadata().ok()?)))
                    })
                    .unzip();

                self.entries = paths;
                self.metadata = metadatas;
            }
            Err(err) => {
                error!("Couldn't open directory: {} Error: {}", path, err)
            }
        }
    }

    fn delete_file(&mut self, file: path::PathBuf) {}

    fn confirm_action(&mut self) -> Result<(), std::io::Error> {
        match self.confirm_action {
            ConfirmAction::Delete(index) => {
                let path = self.entries[index].clone();
                let metadata = self.metadata[index].clone();

                if metadata.is_dir() {
                } else if metadata.is_file() {
                    let result = fs::remove_file(path);
                    self.confirm_action = ConfirmAction::None;
                    self.read_path(self.open_path.clone());
                    return result;
                }
            }

            _ => {}
        }

        self.confirm_action = ConfirmAction::None;
        Ok(())
    }

    fn cancel_action(&mut self) {
        self.confirm_action = ConfirmAction::None
    }
}
