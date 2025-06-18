use std::fs::File;
use std::io::{self, BufReader, BufRead, Write, stdout};
use std::path::Path;
use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::*;
use crossterm::terminal::{Clear, ClearType};

pub enum Mode {
    Insert,
    Command,
}

pub struct Editor {
    pub content: Vec<String>,
    pub cursor_l: usize,
    pub cursor_c: usize,
    pub file_path: String,
    pub mode: Mode,
    pub status_message: String,
    pub command: String,
}

impl Editor {
    pub fn new(file_path: &str) -> Self {
       
        let content = if Path::new(file_path).exists() {
            read_file(file_path)
        } else {
            vec![String::new()]
        };

        Editor {
            content,
            cursor_l: 0,
            cursor_c: 0,
            file_path: file_path.to_string(),
            mode: Mode::Command,
            status_message: String::new(),
            command: String::new(),
        }
    }

    pub fn render(&self) {
        let mut stdout = stdout();

        let (cols, rows) = terminal::size().unwrap();

        let mode_label = match self.mode {
            Mode::Insert => "-- INSERT --",
            Mode::Command => "-- COMMAND --",
        };

        let status = format!("{} | {} | ln {} | col {} | {}", 
            mode_label, 
            self.file_path, 
            self.cursor_l + 1,
            self.cursor_c + 1,
            self.status_message
        );

        queue!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0,0)
        ).unwrap();

        queue!(
            stdout,
            Print("ReEdit"),
            MoveTo(0, 1),
            Print(format!("< {} >", self.file_path)),
            MoveTo(0, 2),
            Print(":w - Save | :q - Exit | i - Insert Mode"),
            MoveTo(0, 3),
            Print("---------------------------------------")
        ).unwrap();

        for (i, line) in self.content.iter().enumerate() {
            queue!(
                stdout,
                MoveTo(0, (i + 5) as u16),
                Clear(ClearType::CurrentLine),
                Print(line)
            ).unwrap();
        }

        queue!(
            stdout,
            MoveTo(0, rows - 1),
            Clear(ClearType::CurrentLine),
            Print(status)
        ).unwrap();

        if matches!(self.mode, Mode::Command) {
            queue!(
                stdout,
                MoveTo(0, rows - 2),
                Clear(ClearType::CurrentLine),
                Print(format!("{}", self.command.to_string()))
            ).unwrap();
        }

        queue!(
            stdout,
            MoveTo(self.cursor_c as u16, (self.cursor_l + 5) as u16)
        ).unwrap();

        stdout.flush().unwrap();
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].insert(self.cursor_c, c);
            self.cursor_c += 1;
        }
    }

    pub fn handle_enter(&mut self) {
        let new_line = if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].split_off(self.cursor_c)
        } else {
            String::new()
        };

        self.content.insert(self.cursor_l + 1, new_line);
        self.cursor_l +=1;
        self.cursor_c = 0;
    }

    pub fn handle_backspace(&mut self) {
        if self.cursor_c > 0 {
            self.content[self.cursor_l].remove(self.cursor_c - 1);
            self.cursor_c -= 1;
        } else if self.cursor_l > 0 {
            let current_line = self.content.remove(self.cursor_l);
            self.cursor_l -= 1;
            self.cursor_c = self.content[self.cursor_l].len();
            self.content[self.cursor_l].push_str(&current_line);
        }
    }

    pub fn handle_delete(&mut self) {
        if self.cursor_l >= self.content.len() {
            return;
        }

        if self.cursor_c < self.content[self.cursor_l].len() {
            self.content[self.cursor_l].remove(self.cursor_c);
        } else if self.cursor_l < self.content.len() - 1 {
            let next_line = self.content.remove(self.cursor_l + 1);
            self.content[self.cursor_l].push_str(&next_line);
        }
    }
    
    pub fn handle_tab(&mut self) {
        if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].insert_str(self.cursor_c, "    ");
            self.cursor_c += 4;
        }
    }

    pub fn auto_close(&mut self, c: char) {
        let close = match c {
            '(' => ')',
            '{' => '}',
            '[' => ']',
            '"' => '"',
            '\'' => '\'',
            _ => { return; }
        };

        if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].insert(self.cursor_c, c);
            self.cursor_c += 1;

            self.content[self.cursor_l].insert(self.cursor_c, close);
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_c < self.content[self.cursor_l].len() {
            self.cursor_c += 1;
        } else if self.cursor_l < self.content.len() -1 {
            self.cursor_l += 1;
            self.cursor_c = 0;
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_c > 0 {
            self.cursor_c -= 1;
        } else if self.cursor_l > 0 {
            self.cursor_l -= 1;
            self.cursor_c = self.content[self.cursor_l].len();
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_l > 0 {
            self.cursor_l -= 1;
            self.cursor_c = std::cmp::min(self.cursor_c, self.content[self.cursor_l].len());
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_l < self.content.len() - 1 {
            self.cursor_l += 1;
            self.cursor_c = std::cmp::min(self.cursor_c, self.content[self.cursor_l].len());
        }
    }

    pub fn save(&mut self) -> io::Result<()> {
        let mut file = File::create(&self.file_path)?;
        file.write_all(self.content.join("\n").as_bytes())?;
        self.status_message = "Saved".to_string();
        Ok(())
    }
}

pub fn read_file(path: &str) -> Vec<String> {
   match File::open(path) {
       Ok(file) => BufReader::new(file).lines().filter_map(Result::ok).collect(),
       Err(_) => vec![String::new()]
   }
}
