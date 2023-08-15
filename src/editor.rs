use std::io::{self, stdout};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct Editor {}

impl Editor {

    pub fn default() -> Self {
        Self {}
    }

    pub fn run(&self) {
        let _stdout = stdout().into_raw_mode().unwrap();

        for key in io::stdin().keys() {
            match key {
                Ok(key) => match key {
                    Key::Char('q') => break,
                    _ => println!("{key:?}"),
                }
                Err(_) => panic!(),
            }
        }
    }
}
