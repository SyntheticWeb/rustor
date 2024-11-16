use crossterm::event::KeyEvent;
use ratatui::layout::Layout;
use ratatui::style::Style;
use ratatui::Frame;

use std::{any::Any, fmt::Debug};

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub title: std::string::String,
    pub version: std::string::String,
}

pub trait App: Any + Debug {
    type Msg: AppMessage;

    fn view(&mut self, layout: &Layout, frame: &mut Frame, style: Style);
    fn update(&mut self, msg: &Self::Msg);
    fn info(&self) -> AppInfo;
    fn generate_msg(&self, key_event: KeyEvent) -> Option<Self::Msg>;
}

pub trait AppMessage: Any {}
