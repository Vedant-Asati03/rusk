use crate::terminal::{GUTTER_WIDTH, Position};
use crate::view::View;
use std::io::{self, stdin};
use termion::{event::Key, input::TermRead};

pub struct Editor {
    should_quit: bool,
    view: View,
    cursor_position: Position,
}

impl Editor {
    pub fn default() -> Result<Self, io::Error> {
        Ok(Self {
            should_quit: false,
            view: View::default()?,
            cursor_position: Position::default(),
        })
    }
}

impl Editor {
    pub fn run(&mut self) -> Result<(), io::Error> {
        loop {
            if let Err(e) = self
                .view
                .render(&mut self.cursor_position, self.should_quit)
            {
                let _ = self.view.quit();
                return Err(e);
            }

            if self.should_quit {
                break;
            }

            if let Err(e) = self.process_keypress() {
                let _ = self.view.quit();
                return Err(e);
            }
        }
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<(), io::Error> {
        let pressed_key = read_key()?;

        match pressed_key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(pressed_key),
            _ => (),
        }
        Ok(())
    }

    fn move_cursor(&mut self, key: Key) {
        let terminal_size = self.view.size();
        let Position { mut x, mut y } = self.cursor_position;

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < terminal_size.height.saturating_sub(1) {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > GUTTER_WIDTH {
                    x = x.saturating_sub(1);
                }
            }
            Key::Right => {
                if x < terminal_size.width.saturating_sub(1) {
                    x = x.saturating_add(1);
                }
            }
            _ => (),
        }

        self.cursor_position = Position { x, y };
    }
}

fn read_key() -> Result<Key, io::Error> {
    loop {
        if let Some(key) = stdin().keys().next() {
            return key;
        }
    }
}
