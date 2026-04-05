use crate::buffer::TextBuffer;
use crate::terminal::{GUTTER_WIDTH, Position};
use crate::view::View;
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use std::{
    fs, io,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};
use termion::{event::Key, input::TermRead};

enum EditorEvent {
    Key(Key),
    Resize,
}

/// Coordinates editing state, input handling, and rendering.
pub struct Editor {
    should_quit: bool,
    view: View,
    cursor_position: Position,
    buffer: TextBuffer,
}

impl Editor {
    /// Creates a new editor session.
    pub fn new() -> Result<Self, io::Error> {
        Ok(Self {
            should_quit: false,
            view: View::new()?,
            cursor_position: Position::default(),
            buffer: TextBuffer::default(),
        })
    }
}

impl Editor {
    /// Opens an optional file and starts processing key events.
    pub fn run(&mut self, filename: &str) -> Result<(), io::Error> {
        if !filename.is_empty() {
            load_file(&mut self.buffer, filename)?;
        }

        let events = Self::spawn_event_channel()?;
        let mut should_render = true;
        let run_result = loop {
            if should_render {
                if let Err(error) =
                    self.view
                        .render(&mut self.cursor_position, self.should_quit, &self.buffer)
                {
                    break Err(error);
                }

                if self.should_quit {
                    break Ok(());
                }

                should_render = false;
            }

            match events.recv() {
                Ok(EditorEvent::Key(key)) => {
                    self.process_keypress(key);
                    should_render = true;
                }
                Ok(EditorEvent::Resize) => {
                    let mut saw_resize = true;

                    while let Ok(event) = events.try_recv() {
                        match event {
                            EditorEvent::Resize => saw_resize = true,
                            EditorEvent::Key(key) => {
                                self.process_keypress(key);
                                should_render = true;
                            }
                        }
                    }

                    if saw_resize {
                        should_render = self.view.refresh_size() || should_render;
                    }
                }
                Err(error) => {
                    break Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        format!("event stream closed: {error}"),
                    ));
                }
            }
        };

        let teardown_result = self.view.teardown();
        match (run_result, teardown_result) {
            (Err(error), _) => Err(error),
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(error)) => Err(error),
        }
    }

    fn spawn_event_channel() -> Result<Receiver<EditorEvent>, io::Error> {
        let (sender, receiver) = mpsc::channel();
        Self::spawn_input_thread(sender.clone());
        Self::spawn_resize_thread(sender)?;
        Ok(receiver)
    }

    fn spawn_input_thread(sender: Sender<EditorEvent>) {
        thread::spawn(move || {
            for key in io::stdin().keys().flatten() {
                if sender.send(EditorEvent::Key(key)).is_err() {
                    break;
                }
            }
        });
    }

    fn spawn_resize_thread(sender: Sender<EditorEvent>) -> Result<(), io::Error> {
        let mut signals = Signals::new([SIGWINCH])
            .map_err(|error| io::Error::other(format!("failed to register SIGWINCH: {error}")))?;

        thread::spawn(move || {
            for _ in signals.forever() {
                if sender.send(EditorEvent::Resize).is_err() {
                    break;
                }
            }
        });

        Ok(())
    }

    /// Handles a single keypress.
    fn process_keypress(&mut self, key: Key) {
        match key {
            Key::Ctrl('q') => self.should_quit = true,
            Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(key),
            Key::Char(c) => {
                let index = self.index_from_cursor_position();
                self.buffer.insert_char(index, c);
                self.set_cursor_from_index(index + 1);
            }
            Key::Backspace => {
                let index = self.index_from_cursor_position();
                if index > 0 {
                    self.buffer.delete_backward(index);
                    self.set_cursor_from_index(index - 1);
                }
            }
            _ => (),
        }
    }

    /// Converts the current cursor location to a character index.
    fn index_from_cursor_position(&self) -> usize {
        let (row, col) = self.get_cursor_position();
        self.buffer.index_from_row_col(row, col)
    }

    /// Positions the cursor at the provided character index.
    fn set_cursor_from_index(&mut self, index: usize) {
        let (row, col) = self.buffer.row_col_from_index(index);
        let (_, terminal_height) = self.view.get_terminal_size();

        let clamped_row = row.min(usize::from(terminal_height));
        let max_col = self.buffer.line_len(clamped_row);
        let clamped_col = col.min(max_col);

        self.cursor_position = Position {
            x: GUTTER_WIDTH.saturating_add(Self::to_u16(clamped_col)),
            y: Self::to_u16(clamped_row),
        };
    }

    /// Returns cursor location as 0-based `(row, col)` excluding gutter width.
    fn get_cursor_position(&self) -> (usize, usize) {
        (
            usize::from(self.cursor_position.y),
            usize::from(self.cursor_position.x.saturating_sub(GUTTER_WIDTH)),
        )
    }

    /// Moves cursor according to arrow key input.
    fn move_cursor(&mut self, key: Key) {
        let (max_visible_col, max_visible_row) = self.view.get_terminal_size();
        let (mut row, mut col) = self.get_cursor_position();

        let total_lines = self.buffer.line_count().saturating_sub(1); // converting to 0-based
        let max_row = total_lines.min(usize::from(max_visible_row));

        match key {
            Key::Up => row = row.saturating_sub(1),
            Key::Down => {
                if row < max_row {
                    row += 1;
                }
            }
            Key::Left => {
                if col > 0 {
                    col -= 1;
                } else if row > 0 {
                    row -= 1;
                    col = self.buffer.line_len(row);
                }
            }
            Key::Right => {
                let line_len = self.buffer.line_len(row);
                if col < line_len {
                    col += 1;
                } else if row < max_row {
                    row += 1;
                    col = 0;
                }
            }
            _ => (),
        }

        let line_len = self.buffer.line_len(row);
        col = col.min(line_len);

        let target_x = GUTTER_WIDTH
            .saturating_add(Self::to_u16(col))
            .min(max_visible_col);

        self.cursor_position = Position {
            x: target_x,
            y: Self::to_u16(row),
        };
    }

    fn to_u16(value: usize) -> u16 {
        value.min(usize::from(u16::MAX)) as u16
    }
}

fn load_file(buffer: &mut TextBuffer, filename: &str) -> Result<(), io::Error> {
    let file_content = fs::read_to_string(filename)?;
    buffer.insert_str(0, &file_content);
    Ok(())
}
