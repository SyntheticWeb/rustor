use crossterm::{
    event::Event,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    ExecutableCommand,
};
use ratatui::{
    crossterm::event::{self, KeyCode},
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

mod app;
mod filetree;
mod mainscreen;
mod rain;

use app::{App, AppInfo};
use filetree::FileTreeApp;
use mainscreen::MainScreenApp;

#[derive(Debug)]
enum AppType {
    MainScreenApp(MainScreenApp),
    FileTreeApp(FileTreeApp),
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
    let mut terminal = init_terminal()?;
    terminal.clear()?;

    let main_screen = MainScreenApp::new();
    let file_tree = FileTreeApp::new();

    let apps = vec![AppType::MainScreenApp(main_screen),AppType::FileTreeApp(file_tree)];

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
        Message::SwapFocus => {}
    }
}

fn view(model: &mut Rustor, frame: &mut Frame) {
    let items: Vec<ListItem> = model
        .apps
        .iter()
        .map(|i| {
            let info: AppInfo;
            match i {
                AppType::FileTreeApp(app)=> info = app.info().clone(),
                AppType::MainScreenApp(app)=> info = app.info().clone(),
            }
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
            match &model.apps[model.selected_app] {
                AppType::FileTreeApp(app)=> app.view(&model.layout, frame),
                AppType::MainScreenApp(app)=> app.view(&model.layout,frame),
            }
    }
}

fn handle_event(model: &mut Rustor) -> io::Result<Option<Message>> {
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
        KeyCode::Tab => Some(Message::SwapFocus),
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
