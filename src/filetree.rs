use ratatui::{layout::Layout, widgets::Paragraph, Frame};

use std::fmt::Debug;

use crate::app::{App, AppInfo};

#[derive(Debug, Clone)]
pub struct FileTreeApp {
    info: AppInfo,
    text: std::string::String,
}

impl App for FileTreeApp {
    fn view(&self, layout: &Layout, frame: &mut Frame) {
        let title_screen = Paragraph::new(String::from(&self.text));
        frame.render_widget(title_screen, layout.split(frame.area())[1]);
    }

    fn update(&mut self) {}

    fn info(&self) -> AppInfo {
        return self.info.clone();
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
