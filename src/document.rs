use crate::Row;
use crate::editor::Position;

#[derive(Default)]
pub struct Document {
    pub rows: Vec<Row>,
    pub filename: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = std::fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        Ok(Self { rows, filename: Some(filename.to_string()), })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn lines(&self) -> usize {
        self.rows.len()
    }

    pub fn begin_insert(&mut self, at: &Position, mode: char) -> Position {
        let mut new_at = Position { x: at.x, y: at.y };
        let current_row = self.row(at.y).unwrap();
        let mut insert_line = false;
        match mode {
            'i' => (),
            'I' => new_at.x = current_row.head(),
            'o' => {
                insert_line = true;
                new_at.y += 1;
            }
            'O' => {
                insert_line = true;
            }
            'a' => new_at.x = at.x + 1,
            'A' => new_at.x = current_row.len(),
            _ => (),
        }
        if insert_line {
            let row = Row::default();
            self.rows.insert(new_at.y, row);
        }
        new_at
    }

    pub fn insert(&mut self, c: char, at: &Position) -> Position {
        let mut new_at = Position { x: at.x, y: at.y };
        let row = self.rows.get_mut(new_at.y).unwrap();
        if c == '\n' {
            let new_row = row.split(new_at.x);
            new_at.y += 1;
            new_at.x = 0;
            self.rows.insert(new_at.y, new_row);
        } else {
            row.insert(new_at.x, c);
            new_at.x += 1;
        }
        new_at
    }

    pub fn backspace(&mut self, at: &Position) -> Position {
        let mut new_at = Position { x: at.x, y: at.y };
        if new_at.x == 0 && new_at.y > 0 {
           //  let prev_row = self.rows.remove(new_at.y - 1);
           //  new_at.y -= 1;
           //  new_at.x = self.rows[new_at.y].len();
           //  self.rows[new_at.y].append(&prev_row);
        } else if new_at.x > 0 {
            self.rows[new_at.y].delete(new_at.x - 1);
            new_at.x -= 1;
        }
        new_at
    }
}
