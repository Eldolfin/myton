use termion::{event::Key, raw::RawTerminal};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{stdin, stdout, Stdout, Write};

const FORBIDENT_REPL_CHARS: &str = "°éèçàù²µù£¤§¨¹̣̣̣̣̣·´¡⅛£$⅜⅝⅞™±°¬¿°¯ˇ˘˙÷×˝";
const PROMPT: &str = ">>> ";

pub struct Repl {
    buffer: Buffer,
    cursor: (u16, u16),
    term_size: (u16, u16),
    input_history: History,
    stdout: RawTerminal<Stdout>,
}

impl Repl {
    pub fn new() -> Repl {
        Repl {
            buffer: Buffer::new(),
            cursor: (1, 1),
            term_size: termion::terminal_size().unwrap(),
            input_history: History::new(),
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    pub fn welcome_prompt(&mut self) {
        self.clear_all();
        self.println("Myton 0.0.1 (main) [Rust 1.65.0] on linux".to_string());
        self.println("Type \"help\" for more information.".to_string());
    }

    fn update_cursor(&mut self) {
        self.cursor.0 = (self.buffer.cursor + PROMPT.len() + 1) as u16;
        print!("{}", termion::cursor::Goto(self.cursor.0, self.cursor.1));
        self.flush();
    }

    fn exit(&mut self) {
        self.clear_all();
        self.println("Byebye!".to_string());
        return;
    }

    fn update_buffer(&mut self) {
        self.clear_line();
        self.print(self.buffer.buffer.clone());
    }

    fn clear_line(&mut self) {
        let prompt_len = PROMPT.len() as u16;
        print!("{}{}{}",
            termion::cursor::Goto(prompt_len, self.cursor.1),
            termion::clear::AfterCursor,
            termion::cursor::Goto(prompt_len, self.cursor.1)
        );
        self.cursor.0 = prompt_len + 1;
    }
    
    fn execute_buffer(&mut self) {
        self.newline();
        self.update_cursor();
        if self.buffer.buffer.len() > 0 {
            self.input_history.push(self.buffer.buffer.clone());
        }
        self.buffer.clear();
    }

    fn prompt(&mut self) {
        self.cursor.0 = 1;
        self.print(PROMPT.to_string());
    }

    fn newline(&mut self) {
        self.cursor = (1, (self.cursor.1 + 1) % self.term_size.1);
    }

    fn clear_all(&mut self) {
        print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
        self.cursor = (1, 1);
    }

    fn flush(&mut self) {
        self.stdout.flush().unwrap();
    }

    pub fn print_result(&mut self, result: Result<String, String>) {
        match result {
            Ok(value) => {
                self.println(value);
            },
            Err(error) => {
                self.printerr(error);
            }
        }
    }

    fn print(&mut self, s: String) {
        print!("{}{}", termion::cursor::Goto(self.cursor.0, self.cursor.1 ), s);
        self.cursor.0 += s.len() as u16;
        self.flush();
    }
    
    pub fn println(&mut self, s: String) {
        for line in s.lines() {
            self.print(line.to_string());
            self.newline();
        }
    }

    pub fn printerr(&mut self, s: String) {
        print!("{}", termion::color::Fg(termion::color::Red));
        self.println(s);
        print!("{}", termion::color::Fg(termion::color::Reset));
    }
    
    pub fn skiplines(&mut self, n: u16) {
        self.cursor.1 = (self.cursor.1 + n) % self.term_size.1;
        self.update_cursor();
    }
}

impl Iterator for Repl {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        self.prompt();
        for c in stdin().keys() {
            match c.unwrap() {
                Key::Up => {
                    if let Some(s) = self.input_history.up() {
                        self.buffer.replace(s);
                    }
                },
                Key::Down => {
                    if let Some(s) = self.input_history.down() {
                        self.buffer.replace(s);
                    }
                },
                Key::Left => {
                    self.buffer.left();
                },
                Key::Right => {
                    self.buffer.right();
                },
                Key::Backspace => {
                    self.buffer.backspace();
                },
                Key::Char('\n') => {
                    self.execute_buffer();
                    return Some(self.buffer.buffer.clone());
                },
                Key::Char(c) => {
                    if !FORBIDENT_REPL_CHARS.contains(c) {
                        self.buffer.insert(c);
                    }
                },
                Key::Ctrl('c') => {
                    self.buffer.clear();
                    self.input_history.reset();
                },
                Key::Ctrl('d') => {
                    if self.buffer.is_empty() {
                        self.exit();
                        return None;
                    } else {
                        self.buffer.clear();
                        self.input_history.reset();
                    }
                },
                Key::Ctrl('a') => {
                    self.buffer.home();
                },
                _ => {}
            }
            self.update_buffer();
            self.update_cursor();
        }
        None
    }
}

struct Buffer {
    pub buffer: String,
    pub cursor: usize,
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            buffer: String::new(),
            cursor: 0,
        }
    }

    fn insert(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += 1;
    }

    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.buffer.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.cursor += 1;
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    fn replace(&mut self, s: String) {
        self.buffer = s;
        self.cursor = self.buffer.len();
    }

    fn is_empty(&self) -> bool {
        self.buffer.len() == 0
    }

    fn home(&mut self) {
        self.cursor = 0;
    }
}

struct History {
    pub history: Vec<String>,
    pub index: usize,
}

impl History {
    fn new() -> History {
        History {
            history: Vec::new(),
            index: 0,
        }
    }

    fn push(&mut self, s: String) {
        if &s != self.history.last().unwrap_or(&String::new()) {
            self.history.push(s);
            self.index = self.history.len();
        }
        self.reset();
    }

    fn up(&mut self) -> Option<String> {
        if self.index > 0 {
            self.index -= 1;
            Some(self.history[self.index].clone())
        } else {
            None
        }
    }

    fn down(&mut self) -> Option<String> {
        if self.index < self.history.len() {
            self.index += 1;
            if self.index == self.history.len() {
                Some(String::new())
            } else {
                Some(self.history[self.index].clone())
            }
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.index = self.history.len();
    }
}
