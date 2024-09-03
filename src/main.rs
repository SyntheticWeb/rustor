use crossterm::{
    cursor, execute,
    style::{self, style, Color, SetForegroundColor, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::collections::HashMap;
use std::io::{self, Stdout, Write};
use std::{thread, time};

struct Screen {
    width: u16,
    height: u16,
    panes: Vec<Pane>,
}

#[derive(PartialEq)]
struct Pane {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    show: bool,
    element: ElementType,
}

#[derive(PartialEq)]
struct MenuItem {
    text: String,
    order: u16,
    pane: Pane,
}

impl MenuItem {
    fn new(text: String, order: u16, pane: Pane) -> MenuItem {
        MenuItem {
            text: text,
            order: order,
            pane: pane,
        }
    }
}

#[derive(PartialEq)]
struct Menu {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    options: Vec<MenuItem>,
    selected: usize,
}

impl Menu {
    fn new(x: u16, y: u16, w: u16, h: u16, options: Vec<MenuItem>) -> Menu {
        Menu {
            x: 2 * x + 1,
            y: y + 1,
            w: 2 * w - 1,
            h: h - 1,
            options: options,
            selected: 0,
        }
    }

    fn draw(&self, out: &mut Stdout) -> io::Result<()> {
        for item in &self.options {
            if &self.options[self.selected] == item {
                out.execute(style::SetBackgroundColor(Color::DarkGreen))?;
            } else {
                out.execute(style::SetBackgroundColor(Color::Reset))?;
            }

            out.execute(cursor::MoveTo(self.x, self.y + item.order))?;
            out.execute(style::Print(&item.text))?;
        }
        out.execute(style::SetBackgroundColor(Color::Reset))?;
        Ok(())
    }

    fn close(out: &mut Stdout, screen: &mut Screen) -> io::Result<()> {
        Ok(())
    }
}

#[derive(PartialEq)]
enum ElementType {
    Menu(Menu),
    Empty(Empty),
}

#[derive(PartialEq)]
struct Empty {}

impl Pane {
    fn new(x: u16, y: u16, width: u16, height: u16, element: ElementType) -> Pane {
        let pane = Pane {
            x: 2 * x,
            y,
            width: 2 * width,
            height,
            show: false,
            element,
        };

        return pane;
    }

    fn draw(&self, out: &mut Stdout) -> io::Result<()> {
        let xp = self.x;
        let yp = self.y;
        let w = self.width; // We double the effect as it takes 2 horizontal characters to make up the same size as a vertical one
        let h = self.height;

        for x in xp + 1..xp + w {
            out.queue(cursor::MoveTo(x, yp))?.queue(style::Print("─"))?;
            out.flush()?;
        }

        out.queue(cursor::MoveTo(xp + w, yp))?
            .queue(style::Print("╮"))?;

        for y in yp + 1..yp + h {
            out.queue(cursor::MoveTo(xp, y))?.queue(style::Print("│"))?;
            out.flush()?;
        }
        out.queue(cursor::MoveTo(xp + w, yp + h))?
            .queue(style::Print("╯"))?;

        for x in (xp + 1..xp + w).rev() {
            out.queue(cursor::MoveTo(x, yp + h))?
                .queue(style::Print("─"))?;
            out.flush()?;
        }
        out.queue(cursor::MoveTo(xp, yp + h))?
            .queue(style::Print("╰"))?;

        for y in (yp + 1..yp + h).rev() {
            out.queue(cursor::MoveTo(xp + w, y))?
                .queue(style::Print("│"))?;
            out.flush()?;
        }
        out.queue(cursor::MoveTo(xp, yp))?
            .queue(style::Print("╭"))?;

        out.flush()?;

        match &self.element {
            ElementType::Menu(menu) => {
                menu.draw(out)?;
            }
            ElementType::Empty(_empty) => {}
        }

        Ok(())
    }
}

impl Screen {
    fn new(width: u16, height: u16) -> Screen {
        let panes: Vec<Pane> = Vec::new();

        let screen = Screen {
            width,
            height,
            panes,
        };
        return screen;
    }

    fn intro(&self, out: &mut Stdout) -> io::Result<()> {
        let pause = time::Duration::from_millis(1);

        out.queue(cursor::Hide)?;

        execute!(out, SetForegroundColor(Color::Green))?;

        for x in 1..self.width - 1 {
            out.queue(cursor::MoveTo(x, 1))?.queue(style::Print("─"))?;
            out.flush()?;
            thread::sleep(pause);
        }

        out.queue(cursor::MoveTo(self.width - 1, 1))?
            .queue(style::Print("╮"))?;

        for y in 2..self.height - 1 {
            out.queue(cursor::MoveTo(self.width - 1, y))?
                .queue(style::Print("│"))?;
            out.flush()?;
            thread::sleep(pause);
        }
        out.queue(cursor::MoveTo(self.width - 1, self.height - 1))?
            .queue(style::Print("╯"))?;

        for x in (1..self.width - 1).rev() {
            out.queue(cursor::MoveTo(x, self.height - 1))?
                .queue(style::Print("─"))?;
            out.flush()?;
            thread::sleep(pause);
        }
        out.queue(cursor::MoveTo(1, self.height - 1))?
            .queue(style::Print("╰"))?;

        for y in (2..self.height - 1).rev() {
            out.queue(cursor::MoveTo(1, y))?.queue(style::Print("│"))?;
            out.flush()?;
            thread::sleep(pause);
        }
        out.queue(cursor::MoveTo(1, 1))?.queue(style::Print("╭"))?;

        out.flush()?;
        Ok(())
    }

    fn draw_panes(&self, out: &mut Stdout) -> io::Result<()> {
        for pane in &self.panes {
            pane.draw(out)?;
        }

        Ok(())
    }

    fn add_pane(&mut self, pane: Pane) {
        self.panes.push(pane);
    }
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    let (width, height) = terminal::size()?;

    let mut screen = Screen::new(width, height);

    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    screen.intro(&mut stdout)?;

    let edit_pane = Pane::new(20, 20, 10, 10, ElementType::Empty(Empty {}));

    let menu_item = MenuItem::new("start editing".to_string(), 1, edit_pane);
    let menu_item2 = MenuItem::new(
        "start inspecting".to_string(),
        2,
        Pane::new(0, 0, 1, 1, ElementType::Empty(Empty {})),
    );

    let menu_options = vec![menu_item, menu_item2];

    let menu = Menu::new(20, 20, 10, 10, menu_options);

    let menu_pane = Pane::new(20, 20, 10, 10, ElementType::Menu(menu));

    screen.add_pane(menu_pane);

    loop {
        screen.draw_panes(&mut stdout)?;
    }
}
