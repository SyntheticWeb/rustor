use ratatui::{layout::{Alignment, Layout}, style::{Style, Stylize}, widgets::{Block, Padding, Paragraph}, Frame};

use std::fmt::Debug;

use crate::app::{App, AppInfo};

#[derive(Debug, Clone)]
pub struct MainScreenApp {
    info: AppInfo,
    text: std::string::String
}

impl App for MainScreenApp {
    fn view(&self, layout: &Layout, frame: &mut Frame) {
        let title_screen = Paragraph::new(String::from(&self.text))
            .block(Block::new().padding(Padding::new(0, 0, layout.split(frame.area())[1].height/2, 0)).border_style(Style::new().blue()))
            .alignment(Alignment::Center);
        frame.render_widget(title_screen, layout.split(frame.area())[1]);
    }

    fn update(&mut self) {}

    fn info(&self) -> AppInfo {
        return self.info.clone();
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
