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
    type Msg: AppMessage;
    type Generator: MsgGenerator<Msg = Self::Msg>;

    fn view(&self, layout: &Layout, frame: &mut Frame);
    fn update(&mut self, msg: &Self::Msg);
    fn info(&self) -> AppInfo;
    fn generator() -> Self::Generator;
}

pub trait AppMessage: Any {}

pub trait MsgGenerator {
    type Msg: AppMessage;
    fn generate_msg(key_event: KeyEvent) -> impl AppMessage;
}
