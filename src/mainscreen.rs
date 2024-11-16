use ratatui::{
    layout::{Alignment, Layout},
    style::{Style, Stylize},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

use crate::app::{App, AppInfo, AppMessage};
use std::fmt::Debug;

use crossterm::event;
use crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct MainScreenApp {
    info: AppInfo,
    text: std::string::String,
}

pub enum MainScreenMsg {
    Placeholder,
}

impl AppMessage for MainScreenMsg {}

impl App for MainScreenApp {
    type Msg = MainScreenMsg;

    fn view(&mut self, layout: &Layout, frame: &mut Frame, style: Style) {
        let title_screen = Paragraph::new(String::from(&self.text))
            .block(
                Block::new()
                    .padding(Padding::new(
                        0,
                        0,
                        layout.split(frame.area())[1].height / 2,
                        0,
                    ))
                    .border_style(Style::new().blue()),
            )
            .alignment(Alignment::Center);
        frame.render_widget(title_screen, layout.split(frame.area())[1]);
    }

    fn update(&mut self, msg: &Self::Msg) {
        match msg {
            MainScreenMsg::Placeholder => self.text = "Event Received!".to_string(),
        }
    }

    fn info(&self) -> AppInfo {
        return self.info.clone();
    }

    fn generate_msg(&self, key_event: event::KeyEvent) -> Option<Self::Msg> {
        match key_event.code {
            KeyCode::Enter => Some(MainScreenMsg::Placeholder),
            _ => None,
        }
    }
}

impl MainScreenApp {
    pub fn new() -> MainScreenApp {
        let main_app = MainScreenApp {
            info: AppInfo {
                title: "Main Screen".to_string(),
                version: "v1.0".to_string(),
            },
            text: "Welcome to Rustor".to_string(),
        };
        return main_app;
    }
}
