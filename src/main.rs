mod buffer;
mod editor;
mod terminal;
mod view;

use editor::Editor;
use std::env;
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let filename = match args.get(1) {
        Some(name) => name,
        None => "",
    };

    let mut editor = Editor::default()?;
    editor.run(filename)?;
    Ok(())
}
