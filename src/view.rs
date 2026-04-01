use crate::terminal::{Position, Size, Terminal};
use std::io;

pub struct View {
    terminal: Terminal,
}

impl View {
    pub fn default() -> Result<Self, io::Error> {
        Ok(Self {
            terminal: Terminal::default()?,
        })
    }
}

impl View {
    pub fn size(&self) -> Size {
        self.terminal.size()
    }

    pub fn render(
        &mut self,
        cursor_pos: &mut Position,
        should_quit: bool,
    ) -> Result<(), io::Error> {
        self.terminal.cursor_hide()?;
        self.terminal.set_cursor_position(&Position::origin())?;

        if should_quit {
            self.terminal.clear_screen()?;
        } else {
            self.draw_rows(cursor_pos)?;
            self.terminal.set_cursor_position(cursor_pos)?;
        }

        self.terminal.cursor_show()?;
        self.terminal.flush()
    }

    fn draw_rows(&mut self, cursor_pos: &Position) -> Result<(), io::Error> {
        let height = self.terminal.size().height;

        for row in 0..height {
            self.terminal.clear_current_line()?;

            let line = if row == cursor_pos.y {
                format!("{:>2}", row + 1)
            } else {
                format!(" ~")
            };

            self.terminal.print(&line)?;

            if row + 1 < height {
                self.terminal.print("\r\n")?;
            }
        }
        Ok(())
    }

    pub fn quit(&mut self) -> Result<(), io::Error> {
        self.terminal.clear_screen()?;
        self.terminal.set_cursor_position(&Position::origin())?;
        self.terminal.flush()
    }
}
