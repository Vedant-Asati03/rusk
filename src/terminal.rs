use std::io::{self, Write, stdout};
use termion::{
    clear, cursor,
    raw::{IntoRawMode, RawTerminal},
    terminal_size,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    stdout: RawTerminal<std::io::Stdout>,
}

impl Terminal {
    pub fn default() -> Result<Self, io::Error> {
        let (width, height) = terminal_size()?;
        Ok(Self {
            size: Size { width, height },
            stdout: stdout().into_raw_mode()?,
        })
    }
}

impl Terminal {
    pub fn size(&self) -> Size {
        self.size
    }

    pub fn print(&mut self, string: &str) -> Result<(), io::Error> {
        write!(self.stdout, "{}", string)
    }

    pub fn clear_screen(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", clear::All)
    }

    pub fn clear_current_line(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", clear::CurrentLine)
    }

    pub fn set_cursor_position(&mut self, position: &Position) -> Result<(), io::Error> {
        let x = position.x.saturating_add(1);
        let y = position.y.saturating_add(1);
        write!(self.stdout, "{}", cursor::Goto(x, y))
    }

    pub fn cursor_hide(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", cursor::Hide)
    }

    pub fn cursor_show(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", cursor::Show)
    }

    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }
}
