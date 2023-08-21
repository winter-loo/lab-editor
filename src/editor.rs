use crate::Document;
use crate::Row;
use crate::Terminal;
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
#[derive(Copy, Clone, Debug)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

enum Mode {
    Command,
    Search,
    Visual,
    Normal,
    Insert,
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    // track cursor position in document
    cursor_position: Position,
    // distance from document head to viewport head
    offset: Position,
    doc: Document,
    prompt_line: String,
    mode: Mode,
    g_cmd_start: bool,
    prev_cursor_positions: Vec<Position>,
}

impl Editor {
    pub fn default() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let doc = if args.len() > 1 {
            let filename = &args[1];
            Document::open(filename).unwrap_or_default()
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::new(),
            cursor_position: Position::default(),
            offset: Position::default(),
            doc,
            prompt_line: String::from(""),
            mode: Mode::Normal,
            g_cmd_start: false,
            prev_cursor_positions: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(e) = self.process_keypress() {
                die(e);
            }
        }
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    fn draw_row(&self, row: &Row) {
        let start = self.offset.x;
        let end = self.offset.x + self.terminal.size().width as usize;
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.doc.row(row as usize + self.offset.y) {
                self.draw_row(row);
            } else if self.doc.is_empty() && row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());

        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodbye.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_prompt_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }

        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_prompt_bar(&self) {
        Terminal::clear_current_line();
        print!("{}", self.prompt_line);
    }

    fn draw_status_bar(&self) {
        let mut status;
        let mut filename = "[No Name]".to_string();
        let width = self.terminal.size().width as usize;
        if let Some(name) = &self.doc.filename {
            filename = name.clone();
            filename.truncate(20);
        }
        status = format!("{}", filename);
        let line_indicator = format!(
            "{}:{}",
            self.cursor_position.y.saturating_add(1),
            self.cursor_position.x.saturating_add(1),
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_bg_color();
        Terminal::reset_fg_color();
    }

    fn in_command_mode(&mut self, pressed_key: Key) {
        match pressed_key {
            Key::Char('\n') => {
                if self.prompt_line == ":q" {
                    self.should_quit = true;
                } 
                self.prompt_line = "".to_string();
                self.mode = Mode::Normal;
                self.cursor_position = self.prev_cursor_positions.pop().unwrap_or_default();
            }
            Key::Backspace => {
                self.prompt_line.pop();
                let Position {x, ..} = &mut self.cursor_position;
                *x = x.saturating_sub(1);
            }
            Key::Char(c) => {
                self.prompt_line.push(c);
                self.cursor_position.x += 1;
            }
            _ => (),
        };
    }

    fn in_search_mode(&self, _pressed_key: Key) {
        println!("In search mode");
    }

    fn in_visual_mode(&self, _pressed_key: Key) {
        println!("In visual mode");
    }

    fn switch_to_insert_mode(&mut self, pressed_key: Key) {
        self.mode = Mode::Insert;
        self.prompt_line = "-- INSERT --".to_string();
        if let Key::Char(s) = pressed_key {
            let new_pos = self.doc.begin_insert(&self.cursor_position, s);
            self.prompt_line = format!("{:?}", new_pos);
            self.move_cursor(&new_pos);
        }
    }

    fn move_cursor(&mut self, pos: &Position) {
        let Position { mut x, mut y } = pos;
        let vw = self.terminal.size().width as usize;
        let vh = self.terminal.size().height as usize;
        let offset = &mut self.offset;
        let extra_move = match self.mode {
            Mode::Insert => 1,
            _ => 0,
        };

        if y >= self.doc.lines().saturating_add(extra_move) {
            y = self.doc.lines().saturating_add(extra_move).saturating_sub(1);
        }
        if y >= vh {
            offset.y = y - vh + 1;
        } else {
            offset.y = 0;
        }

        let row = self.doc.row(y).unwrap();
        if x >= row.len().saturating_add(extra_move) {
            x = row.len().saturating_add(extra_move).saturating_sub(1);
        }
        if x >= vw {
            offset.x = x - vw + 1;
        } else {
            offset.x = 0;
        }

        self.cursor_position = Position { x, y };
    }

    fn in_normal_mode(&mut self, pressed_key: Key) {
        let Position { x, y } = self.cursor_position;
        match pressed_key {
            Key::Char('i') | Key::Char('I') |
            Key::Char('a') | Key::Char('A') |
            Key::Char('o') | Key::Char('O') |
            Key::Char('r') | Key::Char('R') => {
                self.switch_to_insert_mode(pressed_key);
            }
            Key::Char(':') => {
                self.prompt_line = ":".to_string();
                self.prev_cursor_positions.push(self.cursor_position.clone());
                self.cursor_position = Position { x: 1, y: self.terminal.prompt_line() };
                self.mode = Mode::Command;
            }
            Key::Char('?') | Key::Char('/') => {
                self.mode = Mode::Search;
            }
            Key::Ctrl('v') | Key::Char('v') => {
                self.mode = Mode::Visual;
            }
            Key::Char('$') => {
                let row_len = if let Some(row) = self.doc.row(y) {
                    row.len()
                } else {
                    0
                };
                let new_pos = Position {
                    x: row_len.saturating_sub(1),
                    y,
                };
                self.move_cursor(&new_pos);
            }
            Key::Char('0') => {
                let new_pos = Position {
                    x: 0,
                    y: y,
                };
                self.move_cursor(&new_pos);
            }
            Key::Char('G') => {
                let new_pos = Position {
                    x,
                    y: self.doc.lines() - 1,
                };
                self.move_cursor(&new_pos);
            }
            Key::Char('g') => {
               if self.g_cmd_start {
                   self.g_cmd_start = false;
                   let new_pos = Position {
                       x,
                       y: 0,
                   };
                   self.move_cursor(&new_pos);
               } else {
                   self.g_cmd_start = true;
               } 
            }
            Key::Char('h') => {
                let new_pos = Position {
                    x: x.saturating_sub(1),
                    y,
                };
                self.move_cursor(&new_pos);
            }
            Key::Char('l') => {
                let new_pos = Position {
                    x: x.saturating_add(1),
                    y,
                };
                self.move_cursor(&new_pos);
            }
            Key::Char('j') => {
                let new_pos = Position {
                    x,
                    y: y.saturating_add(1),
                };
                self.move_cursor(&new_pos);
            }
            Key::Char('k') => {
                let new_pos = Position {
                    x,
                    y: y.saturating_sub(1),
                };
                self.move_cursor(&new_pos);
            }
            _ => (),
        }
    }

    fn in_insert_mode(&mut self, pressed_key: Key) {
        match pressed_key {
            Key::Char(c) => {
                let at = Position {
                    x: self.cursor_position.x + self.offset.x,
                    y: self.cursor_position.y + self.offset.y,
                };
                let new_pos = self.doc.insert(c, &at);
                self.move_cursor(&new_pos);

                self.prompt_line = format!("inserted at {:?}, next at {:?}, curosr: {:?}",
                                           at, new_pos, self.cursor_position);
            }
            Key::Backspace => {
                let new_pos = self.doc.backspace(&self.cursor_position);
                self.cursor_position = new_pos;
                self.prompt_line = format!("backspaced at {:?}, next at {:?}", self.cursor_position, new_pos);
            }
            Key::Left => (),
            Key::Right => (),
            Key::Up => (),
            Key::Down => (),
            Key::PageUp => (),
            Key::PageDown => (),
            Key::Esc => {
                self.prompt_line = String::from("");
                self.mode = Mode::Normal;
                self.move_cursor(&Position {
                    x: self.cursor_position.x - 1,
                    y: self.cursor_position.y,
                });
            }
            _ => (),
        }
    }

    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;

        match self.mode {
            Mode::Normal => {
                self.in_normal_mode(pressed_key);
            }
            Mode::Insert => {
                self.in_insert_mode(pressed_key);
            }
            Mode::Command => {
                self.in_command_mode(pressed_key);
            }
            Mode::Search => {
                self.in_search_mode(pressed_key);
            }
            Mode::Visual => {
                self.in_visual_mode(pressed_key);
            }
        }

        Ok(())
    }
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
