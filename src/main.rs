mod buffer;
mod editor;
mod terminal;
mod view;

use editor::Editor;
use std::io;

fn main() -> io::Result<()> {
    let mut editor = Editor::default()?;
    editor.run()?;
    Ok(())
}
