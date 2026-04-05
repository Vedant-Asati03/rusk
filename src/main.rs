mod buffer;
mod editor;
mod terminal;
mod view;

use editor::Editor;
use std::env;
use std::io;

fn main() -> io::Result<()> {
    let filename = env::args().nth(1).unwrap_or_default();

    let mut editor = Editor::new()?;
    editor.run(&filename)?;
    Ok(())
}
