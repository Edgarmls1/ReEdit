use std::fs::{self, File};
use std::env;
use std::io::{self, BufReader, BufRead, Write, stdout};
use std::path::{Path, PathBuf};
use crossterm::cursor::MoveTo;
use crossterm::style::{Print, ResetColor, SetForegroundColor, SetBackgroundColor};
use crossterm::*;
use crossterm::terminal::{self, Clear, ClearType};

const SIDEBAR: f32 = 0.2;

pub enum Mode {
    Insert,
    Command,
    Visual,
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

    pub clipboard: Option<String>,
    pub visual_start: Option<usize>,
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
            clipboard: None,
            visual_start: None,
        }
    }

    pub fn render(&mut self) {
        let mut stdout = stdout();

        let (cols, rows) = terminal::size().unwrap();

        let sidebar_width = (SIDEBAR * cols as f32).floor() as u16;

        let cabecalho1 = "-".repeat(cols.into());
        let cabecalho2 = "-".repeat((cols - sidebar_width).into());
        
        let available_rows = (rows - 8) as usize;

        let mode_label = match self.mode {
            Mode::Insert => "-- INSERT --",
            Mode::Command => "-- COMMAND --",
            Mode::Visual => "-- VISUAL --",
        };

        let status_color = match self.mode {
            Mode::Command => style::Color::Red,
            Mode::Insert => style::Color::Green,
            Mode::Visual => style::Color::Purple,
        };

        let file_name = if self.file_path == "." {
            String::from("Empty File")
        } else {
            relative_path(&self.current_dir, &self.file_path)
        };

        let icon = if self.file_path == "." {
            "📄"
        } else {
            file_icon(&self.file_path)
        };

        let (start, end) = match self.visual_start {
            Some(start) if matches!(self.mode, Mode::Visual) => {
                if start <= self.cursor_l {
                    (start, self.cursor_l)
                } else {
                    (self.cursor_l, start)
                }
            },
            _ => (0, 0),
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
            Print("ReEdit - Terminal Text Editor"),
            MoveTo(0, 1),
            Print(format!("{cabecalho1}")),
            MoveTo(sidebar_width, 3),
            Print(format!("|  < {icon} {file_name} >")),
            MoveTo(sidebar_width, 4),
            Print(format!("{cabecalho2}"))
        ).unwrap();

        for (i, line) in self.content.iter().enumerate().skip(self.scroll_offset).take(available_rows) {
            let y = (i - self.scroll_offset + 6) as u16;
            let is_selected = i >= start && i <= end;

            queue!(
                stdout,
                MoveTo(sidebar_width, y),
                Clear(ClearType::CurrentLine),
                if is_selected {
                    SetBackgroundColor(Color::DarkGrey)
                } else {
                    ResetColor
                },
                if i < 9 {
                    Print(format!("   {}| {}", i + 1, line))
                } else if i < 99 {
                    Print(format!("  {}| {}", i + 1, line))
                } else if i < 999 {
                    Print(format!(" {}| {}", i + 1, line))
                } else {
                    Print(format!("{}| {}", i + 1, line))
                },
                ResetColor
            ).unwrap();
        }

        self.render_file_browser();

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

        let (cols, _) = terminal::size().unwrap();

        let sidebar_width = (SIDEBAR * cols as f32).floor() as u16;

        queue!(
            stdout,
            MoveTo(self.cursor_c as u16 + sidebar_width + 3, (self.cursor_l - self.scroll_offset + 5) as u16)
        ).unwrap();

        for (i, file) in self.files.iter().enumerate() {
            let path = self.current_dir.join(file);
            let display_name = truncate_string(file, sidebar_width.saturating_sub(3).into());
            let icon = if path.is_dir() {
                folder_icon(file)
            } else {
                file_icon(file)
            };

            queue!(
                stdout,
                MoveTo(0, (i + 4) as u16),
                if i == self.file_cursor {
                    SetForegroundColor(style::Color::Green)
                } else {
                    SetForegroundColor(style::Color::White)
                },
                Print(format!("{icon} {display_name}")),
                ResetColor
            ).unwrap();
        }

        queue!(
            stdout,
            MoveTo(0, 2),
            Print(format!("📁 {}", self.current_dir.display()))
        ).unwrap();

        for y in 0..self.files.len() as u16 {
            queue!(
                stdout,
                MoveTo(sidebar_width, y + 4),
                Print("|")
            ).unwrap();
        }
    }

    pub fn draw_cursor(&self) {
        let mut stdout = stdout();
        let (cols, _) = terminal::size().unwrap();

        let sidebar_width = (SIDEBAR * cols as f32).floor() as u16;

        let cursor_char = "";

        let cursor_x = self.cursor_c as u16 + sidebar_width + 6;
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

    pub fn open_file_from_command(&mut self, path_str: &str) {
        let mut path = std::path::PathBuf::from(path_str);

        if path.is_relative() {
            path = self.current_dir.join(path);
        }

        if path.is_file() {
            self.content = read_file(path.to_str().unwrap());
            self.status_message = format!("Opened File: {}", path.display());
        } else {
            self.content = vec![String::new()];
            self.status_message = format!("New File: {}", path.display());
        }

        self.file_path = path.to_str().unwrap().to_string();
        self.cursor_l = 0;
        self.cursor_c = 0;
        self.scroll_offset = 0;
        self.mode = Mode::Insert;
    }

    pub fn insert_char(&mut self, c: char) {
        if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].insert(self.cursor_c, c);
            self.cursor_c += 1;
        }
    }

    pub fn handle_enter(&mut self) {
        if self.cursor_l >= self.content.len() {
            return;
        }

        let current_line = &mut self.content[self.cursor_l];

        let current_indent = current_line
            .chars()
            .take_while(|c| c.is_whitespace())
            .collect::<String>();

        let indent_unit = "    ";
        let current_char = current_line.chars().nth(self.cursor_c);
        let prev_char = if self.cursor_c > 0 {
            current_line.chars().nth(self.cursor_c - 1)
        } else {
            None
        };

        let suffix = current_line.split_off(self.cursor_c);
        self.cursor_l += 1;

        if matches!(
            (prev_char, current_char),
            (Some('{'), Some('}')) |
            (Some('['), Some(']')) |
            (Some('('), Some(')'))
        ) {
            self.content.insert(self.cursor_l, format!("{}{}", current_indent, indent_unit));
            self.content.insert(self.cursor_l + 1, format!("{}{}", current_indent, suffix));
            self.cursor_c = (current_indent.len() + indent_unit.len());
        } else {
            self.content.insert(self.cursor_l, format!("{}{}", current_indent, suffix));
            self.cursor_c = current_indent.len();
        }
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

    pub fn copy_selection(&mut self) {
        if let Some(start) = self.visual_start {
            let (start, end) = if start <= self.cursor_l {
                (start, self.cursor_l)
            } else {
                (self.cursor_l, start)
            };
            let lines = self.content[start..=end].join("\n");
            self.clipboard = Some(lines);
            self.status_message = "copied".to_string();
            self.mode = Mode::Command;
            self.visual_start = None;
        }
    }

    pub fn paste_lines(&mut self) {
        if let Some(ref lines) = self.clipboard {
            let split: Vec<String> = lines.lines().map(String::from).collect();
            self.content.splice(self.cursor_l + 1..self.cursor_l + 1, split);
            self.status_message = "pasted".to_string();
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

    pub fn move_up_files(&mut self) {
        if self.file_cursor > 0 {
            self.file_cursor -= 1;
            self.adjust_sidebar_scroll();
        }
    }

    pub fn move_down_files(&mut self) {
        if self.file_cursor + 1 < self.files.len() {
            self.file_cursor += 1;
            self.adjust_sidebar_scroll();
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

    pub fn refresh_sidebar(&mut self) {
        self.files = read_dir_files(&self.current_dir);
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

    pub fn save_as(&mut self, new_path: &str) -> io::Result<()> {
        let mut path = PathBuf::from(new_path);
        if path.is_relative() {
            path = self.current_dir.join(path);
        }

        let mut file = File::create(&path)?;
        file.write_all(self.content.join("\n").as_bytes())?;

        self.file_path = path.to_str().unwrap().to_string();
        self.status_message = format!("Arquivo salvo como: {}", self.file_path);
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

fn file_icon(file_name: &str) -> &str {
    if file_name.ends_with(".rs") {
        "🦀"
    } else if file_name.ends_with(".go") {
        "🐹"
    } else if file_name.ends_with(".c") {
        "C"
    } else if file_name.ends_with(".cpp") {
        "C++"
    } else if file_name.ends_with(".h") {
        "H"
    } else if file_name.ends_with(".py") {
        "🐍"
    } else if file_name.ends_with(".r") {
        "𝐑"
    } else if file_name.ends_with(".js") {
        "JS"
    } else if file_name.ends_with(".ts") {
        "TS"
    } else if file_name.ends_with(".html") {
        "🌐"
    } else if file_name.ends_with(".css") {
        "🎨"
    } else if file_name.ends_with(".md") {
        ""
    } else if file_name.ends_with(".json") {
        "{}"
    } else if file_name.ends_with(".toml") || file_name.ends_with(".yaml") || file_name.ends_with(".conf") || file_name.ends_with(".config") {
        "⚙️"
    } else if file_name.ends_with(".sh") {
        ">_"
    } else if file_name.ends_with(".txt") {
        ""
    } else if file_name.ends_with(".sql") {
        ""
    } else if file_name.ends_with(".java") {
        "☕"
    } else {
        "📄"
    }
}

fn folder_icon(folder_name: &str) -> &str {
    match folder_name {
        "Downloads" => "📥",
        "Desktop" => "🖥️",
        "Documents" | "Documentos" => "📄",
        "Dev" | "dev" => "</>",
        "Projects" | "projects" => "🗂️",
        "Pictures" | "Imagens" => "🖼️",
        "Music" | "Música" => "🎵",
        "Videos" | "Vídeos" => "🎥",
        ".config" => "⚙️",
        ".git" => "🗃️",
        "node_modules" => "📦",
        "target" => "🛠️",
        _ => "📁",
    }
}

fn relative_path(base: &PathBuf, target: &str) -> String {
    let target_path = Path::new(target);

    if let Ok(relative) = target_path.strip_prefix(base) {
        relative.display().to_string()
    } else {
        target_path.display().to_string()
    }
}
