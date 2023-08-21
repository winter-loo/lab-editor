mod document;
mod editor;
mod row;
mod terminal;

pub use document::Document;
use editor::Editor;
pub use row::Row;
use terminal::Terminal;

fn main() {
    let mut editor = Editor::default();
    editor.run();
}
