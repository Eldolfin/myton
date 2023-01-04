use termion::{event::Key, raw::RawTerminal};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{stdin, stdout, Stdin, Stdout};
use super::errors::report_trace;
use super::run;

const FORBIDENT_REPL_CHARS: &str = "°éèçàù²µù£¤§¨¹̣̣̣̣̣·´¡⅛£$⅜⅝⅞™±°¬¿°¯ˇ˘˙÷×˝";

pub struct Repl {
    buffer: Buffer,
    cursor: (u16, u16),
    term_size: (u16, u16),
    input_history: History,
    stdin: Stdin,
    stdout: RawTerminal<Stdout>
}

impl Repl {
    pub fn new() -> Repl {
        Repl {
            buffer: Buffer::new(),
            cursor: (0, 0),
            term_size: termion::terminal_size().unwrap(),
            input_history: History::new(),
            stdin: stdin(),
            stdout: stdout().into_raw_mode().unwrap(),
        }
    }

    pub fn run(&mut self) {
        self.welcome_prompt();
        self.run_loop();
    }

    fn welcome_prompt(&mut self) {
        self.clear_all();
        self.print("Myton 0.0.1 (main) [Rust 1.65.0] on linux".to_string());
        self.print("Type \"help\" for more information.".to_string());
        self.prompt();
    }


    fn run_loop(&mut self) {
        for c in self.stdin.keys() {
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
                Key::Ctrl('c') => {
                    self.buffer.clear();
                },
                Key::Ctrl('d') => {
                    return;
                },
                Key::Char('\n') => {
                    if self.buffer.buffer.len() > 0 {
                        self.input_history.push(self.buffer.buffer.clone());
                    }
                    self.buffer.clear();
                },
                Key::Char(c) => {
                    if !FORBIDENT_REPL_CHARS.contains(c) {
                        self.buffer.insert(c);
                    }
                },
                _ => {}
            }
        }
    }
    
    fn execute_buffer(&mut self) {
        let run_result = run(self.buffer.buffer.clone());
        if let Err(err) = run_result {
            let msg = report_trace(err);
            self.printerr(msg);
        }
        self.buffer.clear();
        self.prompt();
    }

    fn prompt(&mut self) {
        self.print(">>> ".to_string());
    }

    fn newline(&mut self) {
        self.cursor.1 = (self.cursor.1 + 1) % self.term_size.1;
    }

    fn clear_all(&mut self) {
        print!("{}", termion::clear::All);
    }

    
    fn print(&mut self, s: String) {
        for line in s.lines() {
            print!("{}{}", termion::cursor::Goto(1, self.cursor.1 ), line);
            self.newline();
        }
    }

    fn printerr(&mut self, s: String) {
        for line in s.lines() {
            print!("{}{}", termion::cursor::Goto(1, self.cursor.1 ), line);
            self.newline();
        }
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
        self.history.push(s);
        self.index = self.history.len();
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
}



// pub fn run_prompt(){
//     let stdin = stdin();
//     let mut stdout = stdout().into_raw_mode().unwrap();
//     let mut input_buffer = String::new();
//     let mut input_history : Vec<(String, Vec<usize>)> = Vec::new();
//     let mut input_history_index = 0;
//     let mut y_pos = 0;
//     let mut x_pos = 5;
//     let mut buffer_pos = 0;
//     let mut line_breaks: Vec<usize> = vec![0];
//     let term_size = termion::terminal_size().unwrap();
//     let term_height = term_size.1;
//
//     print!("{}{}Myton 0.0.1 (main) [Rust 1.65.0] on linux", termion::clear::All, termion::cursor::Goto(1, y_pos));
//     y_pos += 1;
//     print!("{}Type \"help\" for more information.", termion::cursor::Goto(1, y_pos));
//     y_pos += 1;
//     print!("{}>>> ", termion::cursor::Goto(1, y_pos));
//     stdout.flush().unwrap();
//     for c in stdin.keys() {
//         match c.unwrap() {
//             Key::Up => {
//                 if input_history_index <= 0 {
//                     continue;
//                 }
//
//                 input_history_index -= 1;
//                 input_buffer = input_history[input_history_index].0.clone();
//                 print!("{}{}{}", termion::cursor::Goto(5, y_pos), termion::clear::AfterCursor, input_buffer);
//                 stdout.flush().unwrap();
//                 buffer_pos = input_buffer.len();
//                 x_pos = buffer_pos + 5;
//                 print!("{}", termion::cursor::Goto(5 + buffer_pos , y_pos));
//             },
//             Key::Down => {
//                 if input_history_index >= input_history.len() {
//                     continue;
//                 }
//
//                 input_history_index += 1;
//                 if input_history_index == input_history.len() {
//                     input_buffer.clear();
//                 } else {
//                     input_buffer = input_history[input_history_index].0.clone();
//                 }
//                 print!("{}{}{}", termion::cursor::Goto(5, y_pos), termion::clear::AfterCursor, input_buffer);
//                 x_pos = input_buffer.len() + 5;
//                 stdout.flush().unwrap();
//             },
//             Key::Left => {
//                 if buffer_pos > 0 {
//                     buffer_pos -= 1;
//                     if x_pos == 5 {
//                         y_pos -= 1;
//                         x_pos = term_size.0  - 1;
//                     } else {
//                         x_pos -= 1;
//                     }
//                     print!("{}", termion::cursor::Goto(x_pos , y_pos));
//                     stdout.flush().unwrap();
//                 }
//             },
//             Key::Right => {
//                 if let Ok((x, _)) = termion::cursor::DetectCursorPos::cursor_pos(&mut stdout){
//                     if x < 5 + input_buffer.len()  {
//                         print!("{}", termion::cursor::Right(1));
//                         stdout.flush().unwrap();
//                     }
//                 }
//             },
//             Key::Backspace => {
//                 if x_pos > 5 {
//                     input_buffer.remove((x_pos-5) );
//                     print!("{}{}{}{}", termion::cursor::Goto(5, y_pos), termion::clear::AfterCursor, input_buffer, termion::cursor::Goto(x_pos  -1, y_pos));
//                     stdout.flush().unwrap();
//                 }
//             },
//             Key::Ctrl('c') => {
//                 print!("{}{}", termion::cursor::Goto(5, y_pos), termion::clear::AfterCursor);
//                 input_buffer.clear();
//             },
//             Key::Ctrl('d') => {
//                 return;
//             },
//             Key::Char('\n') => {
//                 if input_buffer.len() > 0 {
//                     input_history.push((input_buffer.clone(), line_breaks.clone()));
//                     line_breaks = vec![0];
//                     x_pos = 5;
//                     buffer_pos = 0;
//                     
//                     input_history_index = input_history.len();
//                 }
//
//                 print!("\n{}", termion::cursor::Goto(1, y_pos));
//                 y_pos = (y_pos + 1) % term_height;
//                 let run_result = run(input_buffer.clone(), "");
//                 if let Err(err) = run_result {
//                     let msg = report_trace(err);
//                     for line in msg.lines() {
//                         print!("{}{}", termion::cursor::Goto(1, y_pos), line);
//                         y_pos = (y_pos + 1) % term_height;
//                     }
//                 }
//                 input_buffer.clear();
//                 print!("{}>>> ", termion::cursor::Goto(1, y_pos));
//             },
//             Key::Char(c) => {
//                 if FORBIDENT_REPL_CHARS.contains(c) {
//                     continue;
//                 }
//
//                 input_buffer.insert((buffer_pos) , c);
//                 buffer_pos += 1;
//                 x_pos += 1;
//                 let last_break = line_breaks.iter().filter(|&&x| x <= buffer_pos).last().unwrap();
//                 let next_break_maybe = line_breaks.iter().filter(|&&x| x > buffer_pos).next();
//                 let next_break = match next_break_maybe {
//                     Some(x) => x.to_owned(),
//                     None => input_buffer.len(),
//                 };
//                 let slice = &input_buffer[*last_break..next_break];
//                 print!("{}{}{}", termion::cursor::Goto(5, y_pos), slice, termion::cursor::Goto(x_pos , y_pos));
//             },
//             _ => {}
//         }
//
//         stdout.flush().unwrap();
//     }
// }
