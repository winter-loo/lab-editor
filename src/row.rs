#[derive(Default)]
pub struct Row {
    string: String,
}

impl From<&str> for Row {
    fn from(s: &str) -> Self {
        Self {
            string: String::from(s),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = std::cmp::min(self.string.len(), end);
        let start = std::cmp::min(start, end);
        self.string.get(start..end).unwrap_or_default().to_string()
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }

    pub fn head(&self) -> usize {
       let first_not_char = self.string.chars().position(|c| c != ' ');
       match first_not_char {
           Some(i) => i,
           None => 0,
       }
    }

    pub fn insert(&mut self, at: usize, c: char) {
        self.string.insert(at, c);
    }

    pub fn delete(&mut self, at: usize) {
        self.string.remove(at);
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut new_row = Self::default();
        let len = self.string.len();
        if at < len {
            new_row.string = self.string.split_off(at);
        }
        new_row
    }
}
