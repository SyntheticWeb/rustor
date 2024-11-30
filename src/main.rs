use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Direction, Layout},
    prelude::{Backend, CrosstermBackend},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState},
    Frame, Terminal,
};

use std::{
    fmt::Debug,
    io::{self, stdout},
    time::Duration,
};

use log::{error, info, warn};
use tui_logger::{init_logger, set_default_level, TuiLoggerWidget};

mod app;
mod filetree;
mod logging;
mod mainscreen;
mod networkscan;
mod rain;

use app::{App, AppInfo};
use filetree::FileTreeApp;
use logging::LoggingApp;
use mainscreen::MainScreenApp;
use networkscan::NetScanApp;

#[derive(Debug)]
enum AppType {
    MainScreenApp(MainScreenApp),
    FileTreeApp(FileTreeApp),
    LoggingApp(LoggingApp),
    NetScan(NetScanApp),
}

#[derive(Debug, Default)]
pub struct Rustor {
    app_select_state: ListState,
    app_open: bool,
    selected_app: usize,
    exit: bool,
    layout: Layout,
    apps: Vec<AppType>,
    app_focused: bool,
}

impl Rustor {
    fn new(apps: Vec<AppType>) -> Rustor {
        let default_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)]);
        let mut default_menu_state = ListState::default();

        Rustor {
            exit: false,
            layout: default_layout,
            apps,
            selected_app: 0,
            app_select_state: default_menu_state,
            app_open: false,
            app_focused: false,
        }
    }
}

#[derive(PartialEq, Clone)]
enum Message {
    NextApp,
    PrevApp,
    OpenApp,
    CloseRustor,
    SwapFocus,
}

fn main() -> io::Result<()> {
    init_logger(log::LevelFilter::Trace).unwrap();
    set_default_level(log::LevelFilter::Debug);

    let mut terminal = init_terminal()?;
    terminal.clear()?;

    let main_screen = MainScreenApp::new();
    let file_tree = FileTreeApp::new();
    let logging_app = LoggingApp::new();
    let netscan_app = NetScanApp::new();

    let apps = vec![
        AppType::MainScreenApp(main_screen),
        AppType::FileTreeApp(file_tree),
        AppType::LoggingApp(logging_app),
        AppType::NetScan(netscan_app),
    ];

    let mut model = Rustor::new(apps);

    while !model.exit {
        terminal.draw(|f| view(&mut model, f))?;

        let mut current_msg = handle_event(&mut model)?;

        if current_msg.is_some() {
            update(&mut model, current_msg.clone().unwrap());
        }
    }

    restore_terminal()?;
    Ok(())
}

fn update(model: &mut Rustor, msg: Message) {
    if !model.app_focused {
        match msg {
            Message::NextApp => model.app_select_state.select_next(),
            Message::PrevApp => model.app_select_state.select_previous(),
            Message::OpenApp => {
                if !model.app_open {
                    model.app_open = true;
                    model.selected_app = model.app_select_state.selected().unwrap();
                } else {
                    model.selected_app = model.app_select_state.selected().unwrap();
                }
            }
            Message::CloseRustor => model.exit = true,
            Message::SwapFocus => {
                model.app_focused = !model.app_focused;
                info!("Focusing application")
            }
        }
    } else {
        match msg {
            Message::SwapFocus => {
                model.app_focused = !model.app_focused;
                info!("Focusing main menu")
            }
            _ => {}
        }
    }
}

fn view(model: &mut Rustor, frame: &mut Frame) {
    let items: Vec<ListItem> = model
        .apps
        .iter()
        .map(|i| {
            let info: AppInfo;
            match i {
                AppType::FileTreeApp(app) => info = app.info().clone(),
                AppType::MainScreenApp(app) => info = app.info().clone(),
                AppType::LoggingApp(app) => info = app.info().clone(),
                AppType::NetScan(app) => info = app.info().clone(),
            }
            ListItem::new(info.title)
        })
        .collect();

    let mut fg_color = Color::White;

    if !model.app_focused {
        fg_color = Color::Green;
    }

    let menu = List::new(items)
        .block(Block::bordered().title("Rustor Apps"))
        .style(Style::default().fg(fg_color))
        .highlight_symbol("*");

    let screen_split = model.layout.split(frame.area());

    frame.render_stateful_widget(menu, screen_split[0], &mut model.app_select_state);

    if model.app_focused {
        fg_color = Color::Green;
    } else {
        fg_color = Color::White;
    }

    let app_style = Style::default().fg(fg_color);

    if model.app_open {
        match &mut model.apps[model.selected_app] {
            AppType::FileTreeApp(app) => app.view(&model.layout, frame, app_style),
            AppType::MainScreenApp(app) => app.view(&model.layout, frame, app_style),
            AppType::LoggingApp(app) => app.view(&model.layout, frame, app_style),
            AppType::NetScan(app) => app.view(&model.layout, frame, app_style),
        }
    }
}

fn handle_event(model: &mut Rustor) -> io::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(model, key));
            }
        }
    }
    Ok(None)
}

fn handle_key(model: &mut Rustor, key: event::KeyEvent) -> Option<Message> {
    if model.app_focused {
        match key.code {
            KeyCode::Tab => Some(Message::SwapFocus),
            _ => match &mut model.apps[model.selected_app] {
                AppType::FileTreeApp(app) => {
                    let msg = app.generate_msg(key);
                    match msg {
                        Some(app_msg) => {
                            app.update(&app_msg);
                        }
                        None => {}
                    }
                    None
                }
                AppType::MainScreenApp(app) => {
                    let msg = app.generate_msg(key);
                    match msg {
                        Some(app_msg) => {
                            app.update(&app_msg);
                        }
                        None => {}
                    }
                    None
                }
                AppType::LoggingApp(app) => {
                    let msg = app.generate_msg(key);
                    match msg {
                        Some(app_msg) => {
                            app.update(&app_msg);
                        }
                        None => {}
                    }
                    None
                }
                AppType::NetScan(app) => {
                    let msg = app.generate_msg(key);
                    match msg {
                        Some(app_msg) => {
                            app.update(&app_msg);
                        }
                        None => {}
                    }
                    None
                }
            },
        }
    } else {
        match key.code {
            KeyCode::Char('k') => Some(Message::PrevApp),
            KeyCode::Char('j') => Some(Message::NextApp),
            KeyCode::Char('q') => Some(Message::CloseRustor),
            KeyCode::Enter => Some(Message::OpenApp),
            KeyCode::Tab => Some(Message::SwapFocus),
            _ => None,
        }
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
