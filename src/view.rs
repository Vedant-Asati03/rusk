use crate::buffer::TextBuffer;
use crate::terminal::{GUTTER_WIDTH, Position, Size, Terminal};
use std::io;

/// Responsible for terminal rendering and cursor clamping.
pub struct View {
    terminal: Terminal,
    current_size: Size,
}

impl View {
    /// Creates a new renderer backed by a terminal in raw mode.
    pub fn new() -> Result<Self, io::Error> {
        let terminal = Terminal::new()?;
        let current_size = terminal.size();
        Ok(Self {
            terminal,
            current_size,
        })
    }
}

impl View {
    /// Updates cached terminal size and returns whether it changed.
    pub fn refresh_size(&mut self) -> bool {
        let latest = self.terminal.size();
        if latest != self.current_size {
            self.current_size = latest;
            true
        } else {
            false
        }
    }

    /// Returns terminal size as 0-based `(max_col, max_row)` coordinates.
    pub fn get_terminal_size(&self) -> (u16, u16) {
        let size = self.current_size;
        (size.width.saturating_sub(1), size.height.saturating_sub(1))
    }

    /// Renders one frame.
    pub fn render(
        &mut self,
        cursor_pos: &mut Position,
        should_quit: bool,
        buffer: &TextBuffer,
    ) -> Result<(), io::Error> {
        self.refresh_size();
        let size = self.current_size;

        self.terminal.cursor_hide()?;
        self.terminal.set_cursor_position(&Position::origin())?;

        if should_quit {
            self.terminal.clear_screen()?;
        } else {
            self.draw_rows(cursor_pos, buffer, size)?;
            self.clamp_cursor_to_buffer(cursor_pos, buffer, size);
            self.terminal.set_cursor_position(cursor_pos)?;
        }

        self.terminal.cursor_show()?;
        self.terminal.flush()
    }

    /// Draws all rows currently visible in the terminal.
    fn draw_rows(
        &mut self,
        cursor_pos: &Position,
        buffer: &TextBuffer,
        size: Size,
    ) -> Result<(), io::Error> {
        let terminal_height = usize::from(size.height);
        let content_width = usize::from(size.width.saturating_sub(GUTTER_WIDTH));
        let mut last_rendered_row = 0usize;
        let mut render_error: Option<io::Error> = None;

        buffer.for_each_visible_line(0, terminal_height, content_width, |row, line| {
            if render_error.is_some() {
                return;
            }

            while last_rendered_row < row {
                let is_last_row = last_rendered_row + 1 == terminal_height;
                if let Err(error) =
                    self.render_filler_row(last_rendered_row, cursor_pos, is_last_row)
                {
                    render_error = Some(error);
                    return;
                }
                last_rendered_row += 1;
            }

            let is_last_row = row + 1 == terminal_height;
            if let Err(error) = self.render_text_row(row, line, is_last_row) {
                render_error = Some(error);
                return;
            }

            last_rendered_row = row + 1;
        });

        if let Some(error) = render_error {
            return Err(error);
        }

        while last_rendered_row < terminal_height {
            let is_last_row = last_rendered_row + 1 == terminal_height;
            self.render_filler_row(last_rendered_row, cursor_pos, is_last_row)?;
            last_rendered_row += 1;
        }

        Ok(())
    }

    /// Ensures cursor coordinates remain valid for current buffer and viewport.
    fn clamp_cursor_to_buffer(&self, cursor_pos: &mut Position, buffer: &TextBuffer, size: Size) {
        let maximum_visible_rows = usize::from(size.height.saturating_sub(1));
        let maximum_visible_cols =
            usize::from(size.width.saturating_sub(1).saturating_sub(GUTTER_WIDTH));

        let max_row = buffer
            .line_count()
            .saturating_sub(1)
            .min(maximum_visible_rows);

        let row = usize::from(cursor_pos.y).min(max_row);
        let max_col = buffer.line_len(row);
        let col = usize::from(cursor_pos.x.saturating_sub(GUTTER_WIDTH))
            .min(max_col)
            .min(maximum_visible_cols);

        cursor_pos.y = Self::to_u16(row);
        cursor_pos.x = GUTTER_WIDTH.saturating_add(Self::to_u16(col));
    }

    /// Clears terminal state before leaving the editor UI.
    pub fn teardown(&mut self) -> Result<(), io::Error> {
        self.terminal.clear_screen()?;
        self.terminal.set_cursor_position(&Position::origin())?;
        self.terminal.flush()
    }

    fn to_u16(value: usize) -> u16 {
        value.min(usize::from(u16::MAX)) as u16
    }

    fn render_text_row(
        &mut self,
        row: usize,
        text: &str,
        is_last_row: bool,
    ) -> Result<(), io::Error> {
        self.terminal.clear_current_line()?;
        let line_number = format!("{:>2} ", row + 1);
        self.terminal.print(&line_number)?;
        self.terminal.print(text)?;
        if is_last_row {
            Ok(())
        } else {
            self.terminal.print("\r\n")
        }
    }

    fn render_filler_row(
        &mut self,
        row: usize,
        cursor_pos: &Position,
        is_last_row: bool,
    ) -> Result<(), io::Error> {
        self.terminal.clear_current_line()?;
        let line = if row == usize::from(cursor_pos.y) {
            format!("{:>2} ", row + 1)
        } else {
            format!("{:>2} ", "~")
        };
        self.terminal.print(&line)?;
        if is_last_row {
            Ok(())
        } else {
            self.terminal.print("\r\n")
        }
    }
}
