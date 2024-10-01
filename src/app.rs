use crossterm::event::KeyEvent;
use ratatui::layout::Layout;
use ratatui::Frame;

use std::{any::Any, fmt::Debug};

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub title: std::string::String,
    pub version: std::string::String,
}

pub trait App: Any + Debug {
    fn view(&self, layout: &Layout, frame: &mut Frame);
    fn update(&mut self);
    fn info(&self) -> AppInfo;
}

pub trait AppMessage {
    fn generate_msg(key_event: KeyEvent) -> impl AppMessage;
}
