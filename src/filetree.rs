use ratatui::{layout::Layout, widgets::Paragraph, Frame};

use std::fmt::Debug;

use crate::app::{App, AppInfo, AppMessage};

use crossterm::event::KeyCode;
use crossterm::event;

#[derive(Debug, Clone)]
pub struct FileTreeApp {
    info: AppInfo,
    text: std::string::String,
}

pub enum FileTreeMsg {
    Placeholder
}

impl AppMessage for FileTreeMsg {}

impl App for FileTreeApp {
    type Msg = FileTreeMsg;
    fn view(&self, layout: &Layout, frame: &mut Frame) {
        let title_screen = Paragraph::new(String::from(&self.text));
        frame.render_widget(title_screen, layout.split(frame.area())[1]);
    }

    fn update(&mut self, msg: &Self::Msg) {
        match msg {
            FileTreeMsg::Placeholder => self.text = "Event Received!".to_string(),
        }
    }

    fn info(&self) -> AppInfo {
        return self.info.clone();
    }


    fn generate_msg(&self, key_event: event::KeyEvent) -> Option<Self::Msg> {
        match key_event.code {
            KeyCode::Enter => Some(FileTreeMsg::Placeholder),
            _ => None,
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
        };
        return main_app;
    }
}
