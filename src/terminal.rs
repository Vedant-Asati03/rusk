use std::io::{self, Write, stdout};
use termion::{
    clear, cursor,
    raw::{IntoRawMode, RawTerminal},
    screen::{AlternateScreen, IntoAlternateScreen},
    terminal_size,
};

/// Width reserved for the line-number gutter.
pub const GUTTER_WIDTH: u16 = 3;

/// Cursor position in editor space.
///
/// Coordinates are 0-based and `x` includes the gutter offset.
#[derive(Debug)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: GUTTER_WIDTH,
            y: 0,
        }
    }
}

impl Position {
    /// Returns the top-left position in 0-based coordinates.
    pub fn origin() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// Terminal dimensions in 1-based terminal units.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

/// Low-level terminal abstraction used by the editor view.
///
/// The terminal is configured in raw mode and alternate screen mode.
pub struct Terminal {
    stdout: AlternateScreen<RawTerminal<std::io::Stdout>>,
}

impl Terminal {
    /// Creates a terminal configured for full-screen rendering.
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            stdout: stdout().into_raw_mode()?.into_alternate_screen()?,
        })
    }

    fn query_size() -> Size {
        match terminal_size() {
            Ok((width, height)) => Size { width, height },
            Err(_) => Size {
                width: 1,
                height: 1,
            },
        }
    }
}

impl Terminal {
    /// Returns the current terminal size.
    ///
    /// This is queried on demand so window resize events are reflected automatically.
    pub fn size(&self) -> Size {
        Self::query_size()
    }

    /// Writes raw text to the terminal output buffer.
    pub fn print(&mut self, string: &str) -> Result<(), io::Error> {
        write!(self.stdout, "{}", string)
    }

    /// Clears the full terminal screen.
    pub fn clear_screen(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", clear::All)
    }

    /// Clears the current line.
    pub fn clear_current_line(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", clear::CurrentLine)
    }

    /// Moves the terminal cursor to a 0-based editor position.
    pub fn set_cursor_position(&mut self, position: &Position) -> Result<(), io::Error> {
        let x = position.x.saturating_add(1);
        let y = position.y.saturating_add(1);
        write!(self.stdout, "{}", cursor::Goto(x, y))
    }

    /// Hides the cursor.
    pub fn cursor_hide(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", cursor::Hide)
    }

    /// Shows the cursor.
    pub fn cursor_show(&mut self) -> Result<(), io::Error> {
        write!(self.stdout, "{}", cursor::Show)
    }

    /// Flushes the output buffer to the terminal.
    pub fn flush(&mut self) -> Result<(), io::Error> {
        self.stdout.flush()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = write!(self.stdout, "{}", cursor::Show);
        let _ = self.stdout.flush();
    }
}
