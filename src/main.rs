use crossterm::{
    event::{Event, KeyEvent},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{self, Constraint, Direction, Layout},
    prelude::{self, Backend, CrosstermBackend},
    style::{Color, Style, Stylize},
    text::ToText,
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget},
    DefaultTerminal, Frame, Terminal,
};

use std::{
    any::Any,
    borrow::Borrow,
    fmt::Debug,
    io::{self, stdout},
    time::Duration,
};

mod app;
mod filetree;
mod mainscreen;
mod rain;

use app::{App, AppInfo};
use filetree::FileTreeApp;
use mainscreen::MainScreenApp;

#[derive(Debug, Default)]
pub struct Rustor {
    app_select_state: ListState,
    app_open: bool,
    exit: bool,
    layout: Layout,
    apps: Vec<Box<dyn App>>,
}

impl Rustor {
    fn new(apps: Vec<Box<dyn App>>) -> Rustor {
        let default_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)]);
        let mut default_menu_state = ListState::default();

        Rustor {
            exit: false,
            layout: default_layout,
            apps,
            app_select_state: default_menu_state,
            app_open: false,
        }
    }
}

#[derive(PartialEq, Clone)]
enum Message {
    NextApp,
    PrevApp,
    OpenApp,
    CloseRustor,
}

fn main() -> io::Result<()> {
    let mut terminal = init_terminal()?;
    terminal.clear()?;

    let main_screen = MainScreenApp::new();
    let file_tree = FileTreeApp::new();

    let apps: Vec<Box<dyn App>> = vec![Box::new(main_screen), Box::new(file_tree)];

    let mut model = Rustor::new(apps);

    while !model.exit {
        terminal.draw(|f| view(&mut model, f))?;

        let mut current_msg = handle_event(&model)?;

        if current_msg.is_some() {
            update(&mut model, current_msg.clone().unwrap());
        }
    }

    restore_terminal()?;
    Ok(())
}

fn update(model: &mut Rustor, msg: Message) {
    match msg {
        Message::NextApp => model.app_select_state.select_next(),
        Message::PrevApp => model.app_select_state.select_previous(),
        Message::OpenApp => model.app_open = true,
        Message::CloseRustor => model.exit = true,
    }
}

fn view(model: &mut Rustor, frame: &mut Frame) {
    let items: Vec<ListItem> = model
        .apps
        .iter()
        .map(|i| {
            let info = i.info().clone();
            ListItem::new(info.title)
        })
        .collect();

    let menu = List::new(items)
        .block(Block::bordered().title("Rustor Apps"))
        .style(Style::default().fg(Color::Green))
        .highlight_symbol("*");

    let screen_split = model.layout.split(frame.area());

    frame.render_stateful_widget(menu, screen_split[0], &mut model.app_select_state);

    if model.app_open {
        let selected = model.app_select_state.selected().unwrap();
        model.apps[selected].view(&model.layout, frame);
    }
}

fn handle_event(_: &Rustor) -> io::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('k') => Some(Message::PrevApp),
        KeyCode::Char('j') => Some(Message::NextApp),
        KeyCode::Char('q') => Some(Message::CloseRustor),
        KeyCode::Enter => Some(Message::OpenApp),
        _ => None,
    }
}

fn init_terminal() -> io::Result<Terminal<impl Backend>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

fn restore_terminal() -> io::Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
