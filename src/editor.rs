use std::fs::{self, File};
use std::env;
use std::io::{self, BufReader, BufRead, Write, stdout};
use std::path::{Path, PathBuf};
use crossterm::cursor::MoveTo;
use crossterm::style::{Print, ResetColor, SetForegroundColor};
use crossterm::*;
use crossterm::terminal::{self, Clear, ClearType};

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

    pub current_dir: PathBuf,
    pub files: Vec<String>,
    pub file_cursor: usize,

    pub scroll_offset: usize,
    pub sidebar_scroll: usize,
}

impl Editor {
    pub fn new(file_path: &str) -> Self {
       
        let content = if file_path == "." {
            vec![String::new()]
        } else if Path::new(file_path).exists() {
            read_file(file_path)
        } else {
            vec![String::new()]
        };

        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let files = read_dir_files(&current_dir);

        Editor {
            content,
            cursor_l: 0,
            cursor_c: 0,
            file_path: file_path.to_string(),
            mode: Mode::Command,
            status_message: String::new(),
            command: String::new(),
            current_dir,
            files,
            file_cursor: 0,
            scroll_offset: 0,
            sidebar_scroll: 0,
        }
    }

    pub fn render(&mut self) {
        let mut stdout = stdout();

        let sidebar_width = 30;

        let (_cols, rows) = terminal::size().unwrap();

        let available_rows = (rows - 8) as usize;

        let mode_label = match self.mode {
            Mode::Insert => "-- INSERT --",
            Mode::Command => "-- COMMAND --",
        };

        let status_color = match self.mode {
            Mode::Command => style::Color::Red,
            Mode::Insert => style::Color::Green,
        };

        let file_name = if self.file_path == "." {
            String::from("Empty File")
        } else {
            self.file_path.clone()
        };

        let status = format!("{} | {} | ln {} | col {} | {}", 
            mode_label, 
            file_name, 
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
            Print("---------------------------------------------------------------------------"),
            MoveTo(sidebar_width, 3),
            Print(format!("|  < {file_name} >")),
            MoveTo(sidebar_width + 3 , 4),
            Print("------------------------------------------")
        ).unwrap();

        for (i, line) in self.content.iter().enumerate().skip(self.scroll_offset).take(available_rows) {
            queue!(
                stdout,
                MoveTo(sidebar_width + 3, (i - self.scroll_offset + 6) as u16),
                Clear(ClearType::CurrentLine),
                Print(line)
            ).unwrap();
        }

        self.render_file_browser();
        
        queue!(
            stdout,
            MoveTo(0, rows - 1),
            Clear(ClearType::CurrentLine),
            SetForegroundColor(status_color),
            Print(status),
            ResetColor
        ).unwrap();

        self.draw_cursor();

        stdout.flush().unwrap();
    }

    pub fn render_file_browser(&mut self) {
        let mut stdout = stdout();

        let sidebar_width = 30;

        let (_cols, rows) = terminal::size().unwrap();

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
            MoveTo(self.cursor_c as u16 + sidebar_width + 3, (self.cursor_l - self.scroll_offset + 5) as u16)
        ).unwrap();

        for (i, file) in self.files.iter().enumerate() {
            let path = self.current_dir.join(file);
            let display_name = truncate_string(file, sidebar_width.saturating_sub(3).into();

            queue!(
                stdout,
                MoveTo(0, (i + 4) as u16),
                if i == self.file_cursor {
                    SetForegroundColor(style::Color::Green)
                } else {
                    SetForegroundColor(style::Color::White)
                },
                if path.is_dir() {
                    Print(format!("📁 {}", display_name))
                } else {
                    Print(format!("📄 {}", display_name))
                },
                ResetColor
            ).unwrap();
        }

        queue!(
            stdout,
            MoveTo(0, 2),
            Print(format!("📁 {}", self.current_dir.display()))
        ).unwrap();

        let (cols, _) = terminal::size().unwrap();
        for y in 0..self.files.len().min(20) as u16 {
            queue!(
                stdout,
                MoveTo(sidebar_width as u16, y + 4),
                Print("|")
            ).unwrap();
        }
    }

    pub fn draw_cursor(&self) {
        let mut stdout = stdout();
        let sidebar_width = 30;
        let cursor_char = "";

        let cursor_x = self.cursor_c as u16 + sidebar_width + 3;
        let cursor_y = (self.cursor_l - self.scroll_offset + 6) as u16;

        queue!(
            stdout,
            MoveTo(cursor_x, cursor_y),
            Print(cursor_char),
        ).unwrap();

        stdout.flush().unwrap();
    }

    pub fn adjust_scroll(&mut self) {
        let (_, rows) = terminal::size().unwrap();
        let available_rows = (rows - 8) as usize;

        if self.cursor_l < self.scroll_offset {
            self.scroll_offset = self.cursor_l;
        } else if self.cursor_l >= self.scroll_offset + available_rows {
            self.scroll_offset = self.cursor_l - available_rows + 1;
        }
    }

    pub fn adjust_sidebar_scroll(&mut self) {
        let (_, rows) = terminal::size().unwrap();
        let visible_files = (rows - 6) as usize;
    
        if self.file_cursor < self.sidebar_scroll {
            self.sidebar_scroll = self.file_cursor;
        } else if self.file_cursor >= self.sidebar_scroll + visible_files {
            self.sidebar_scroll = self.file_cursor - visible_files + 1;
        }
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

    pub fn open_selected(&mut self) {
        if self.files.is_empty() { return; }

        let selected = &self.files[self.file_cursor];
        let path = self.current_dir.join(selected);

        if path.is_file() {
            self.content = read_file(path.to_str().unwrap());
            self.file_path = path.to_str().unwrap().to_string();
            self.cursor_l = 0;
            self.cursor_c = 0;
            self.status_message = format!("Arquivo aberto: {}", selected);
            self.mode = Mode::Insert;
        } else if path.is_dir() {
            self.current_dir = path;
            self.files = read_dir_files(&self.current_dir);
            self.file_cursor = 0;
            self.status_message = format!("Entrou no diretório: {}", self.current_dir.display());
        }
    }

    pub fn go_back(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.files = read_dir_files(&self.current_dir);
            self.file_cursor = 0;
            self.status_message = format!("Diretório: {}", self.current_dir.display());
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor_c < self.content[self.cursor_l].len() {
            self.cursor_c += 1;
            self.adjust_scroll();
        } else if self.cursor_l < self.content.len() -1 {
            self.cursor_l += 1;
            self.cursor_c = 0;
            self.adjust_scroll();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_c > 0 {
            self.cursor_c -= 1;
            self.adjust_scroll();
        } else if self.cursor_l > 0 {
            self.cursor_l -= 1;
            self.cursor_c = self.content[self.cursor_l].len();
            self.adjust_scroll();
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_l > 0 {
            self.cursor_l -= 1;
            self.cursor_c = std::cmp::min(self.cursor_c, self.content[self.cursor_l].len());
            self.adjust_scroll();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_l < self.content.len() - 1 {
            self.cursor_l += 1;
            self.cursor_c = std::cmp::min(self.cursor_c, self.content[self.cursor_l].len());
            self.adjust_scroll();
        }
    }

    pub fn save(&mut self) -> io::Result<()> {
        if self.file_path == "." {
            self.status_message = "Usage :w <file_path>".to_string();
            return Ok(());
        }

        let mut file = File::create(&self.file_path)?;
        file.write_all(self.content.join("\n").as_bytes())?;
        self.status_message = "File Saved".to_string();
        Ok(())
    }
}

pub fn read_file(path: &str) -> Vec<String> {
   match File::open(path) {
       Ok(file) => BufReader::new(file).lines().filter_map(Result::ok).collect(),
       Err(_) => vec![String::new()]
   }
}

pub fn read_dir_files(path: &PathBuf) -> Vec<String> {
    let mut entries: Vec<String> = fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|e| e.file_name().into_string().unwrap_or_default())
        .collect();

    entries.sort();
    entries
}

fn truncate_string(s: &str, max_width: usize) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else if max_width > 1 {
        let truncated: String = s.chars().take(max_width - 1).collect();
        format!("{}…", truncated)
    } else {
        "…".to_string()
    }
}
