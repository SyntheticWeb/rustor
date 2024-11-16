use ratatui::{
    layout::{Alignment, Layout},
    style::{Style, Stylize},
    widgets::{Block, Padding, Paragraph},
    Frame,
};
use tui_logger::TuiLoggerWidget;

use crate::app::{App, AppInfo, AppMessage};
use std::fmt::Debug;

use crossterm::event;
use crossterm::event::KeyCode;

#[derive(Debug, Clone)]
pub struct LoggingApp {
    info: AppInfo,
    text: std::string::String,
}

pub enum LoggingMsg {
    Placeholder,
}

impl AppMessage for LoggingMsg {}

impl App for LoggingApp {
    type Msg = LoggingMsg;

    fn view(&mut self, layout: &Layout, frame: &mut Frame, style: Style) {
        let logger_widget = TuiLoggerWidget::default()
            .block(
                ratatui::widgets::Block::default()
                    .title("Logs")
                    .borders(ratatui::widgets::Borders::ALL),
            )
            .style_error(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
            .style_warn(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
            .style_info(ratatui::style::Style::default().fg(ratatui::style::Color::Blue));

        frame.render_widget(logger_widget, layout.split(frame.area())[1]);
    }

    fn update(&mut self, msg: &Self::Msg) {
        match msg {
            LoggingMsg::Placeholder => self.text = "Event Received!".to_string(),
        }
    }

    fn info(&self) -> AppInfo {
        return self.info.clone();
    }

    fn generate_msg(&self, key_event: event::KeyEvent) -> Option<Self::Msg> {
        match key_event.code {
            KeyCode::Enter => Some(LoggingMsg::Placeholder),
            _ => None,
        }
    }
}

impl LoggingApp {
    pub fn new() -> LoggingApp {
        let log_app = LoggingApp {
            info: AppInfo {
                title: "Logging Screen".to_string(),
                version: "v1.0".to_string(),
            },
            text: "Logging App".to_string(),
        };
        return log_app;
    }
}
