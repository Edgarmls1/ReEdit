use std::fs::File;
use std::io::{self, BufReader, BufRead, Write, stdout};
use std::path::Path;
use std::env;
use crossterm::cursor::MoveTo;
use crossterm::event::{Event, KeyCode, KeyModifiers, KeyEvent};
use crossterm::style::{self, Print};
use crossterm::*;
use crossterm::terminal::{Clear, ClearType};

struct Editor {
    content: Vec<String>,
    cursor_l: usize,
    cursor_c: usize,
    file_path: String
}

impl Editor {
    fn new(file_path: &str) -> Self {
       
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
        }
    }

    fn render(&self) {
        let mut stdout = stdout();

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
            Print("Ctrl+S - Save | Esc - Exit"),
            MoveTo(0, 3),
            Print("----------------------------")
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
            MoveTo(self.cursor_c as u16, (self.cursor_l + 5) as u16)
        ).unwrap();

        stdout.flush().unwrap();
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].insert(self.cursor_c, c);
            self.cursor_c += 1;
        }
    }

    fn handle_enter(&mut self) {
        let new_line = if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].split_off(self.cursor_c)
        } else {
            String::new()
        };

        self.content.insert(self.cursor_l + 1, new_line);
        self.cursor_l +=1;
        self.cursor_c = 0;
    }

    fn handle_backspace(&mut self) {
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
    
    fn handle_tab(&mut self) {
        if self.cursor_l < self.content.len() {
            self.content[self.cursor_l].insert_str(self.cursor_c, "    ");
            self.cursor_c += 4;
        }
    }

    fn auto_close(&mut self, c: char) {
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

    fn move_right(&mut self) {
        if self.cursor_c < self.content[self.cursor_l].len() {
            self.cursor_c += 1;
        } else if self.cursor_l < self.content.len() -1 {
            self.cursor_l += 1;
            self.cursor_c = 0;
        }
    }

    fn move_left(&mut self) {
        if self.cursor_c > 0 {
            self.cursor_c -= 1;
        } else if self.cursor_l > 0 {
            self.cursor_l -= 1;
            self.cursor_c = self.content[self.cursor_l].len();
        }
    }

    fn move_up(&mut self) {
        if self.cursor_l > 0 {
            self.cursor_l -= 1;
            self.cursor_c = std::cmp::min(self.cursor_c, self.content[self.cursor_l].len());
        }
    }

    fn move_down(&mut self) {
        if self.cursor_l < self.content.len() - 1 {
            self.cursor_l += 1;
            self.cursor_c = std::cmp::min(self.cursor_c, self.content[self.cursor_l].len());
        }
    }

    fn save(&self) -> io::Result<()> {
        let mut file = File::create(&self.file_path)?;
        file.write_all(self.content.join("\n").as_bytes())?;
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("usage: reedit <file_path>");
        return Ok(());
    }

    let file_path = &args[1];
    let mut editor = Editor::new(file_path);

    terminal::enable_raw_mode()?;

    let _raw = terminal::enable_raw_mode();

    editor.render();
    loop {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            match (code, modifiers) {
                (KeyCode::Esc, _) => break,
                (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                    editor.save()?;
                    println!("\n Arquivo salvo com sucesso");
                },
                (KeyCode::Enter, _) => editor.handle_enter(),
                (KeyCode::Backspace, _) => editor.handle_backspace(),
                (KeyCode::Left, _) => editor.move_left(),
                (KeyCode::Right, _) => editor.move_right(),
                (KeyCode::Up, _) => editor.move_up(),
                (KeyCode::Down, _) => editor.move_down(),
                (KeyCode::Char(c), _) => {
                    match c {
                        '(' | '[' | '{' | '"' | '\'' => editor.auto_close(c),
                        _ => editor.insert_char(c),
                    }
                },
                (KeyCode::Tab, _) => editor.handle_tab(), 
                (_,_) => {}
            }
            editor.render();
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}

fn read_file(path: &str) -> Vec<String> {
   match File::open(path) {
       Ok(file) => BufReader::new(file).lines().filter_map(Result::ok).collect(),
       Err(_) => vec![String::new()]
   }
}
