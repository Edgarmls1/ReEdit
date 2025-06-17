mod editor;

use std::env;
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;

use editor::Editor;

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

    loop {
        editor.render();
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            match (code, modifiers) {
                (KeyCode::Esc, _) => {
                    editor.mode = editor::Mode::Command;
                },
                (KeyCode::Char('i'), _) if matches!(editor.mode, editor::Mode::Command) => {
                    editor.mode = editor::Mode::Insert;
                },
                (KeyCode::Enter, _) if matches!(editor.mode, editor::Mode::Command) => {
                    match editor.command.as_str() {
                        ":w" => { 
                            editor.save()?; 
                            editor.status_message = "File Saved".to_string(); 
                        },
                        ":q" => { 
                            break; 
                        },
                        ":wq" => { 
                            editor.save()?; 
                            break; 
                        },
                        _ => {
                            editor.status_message = "Unknow Command".to_string();
                        }
                    }
                    editor.command.clear();
                },
                (KeyCode::Backspace, _) if matches!(editor.mode, editor::Mode::Command) => {
                    editor.command.pop();
                },
                (KeyCode::Char(c), _) if matches!(editor.mode, editor::Mode::Command) => {
                    editor.command.push(c);
                },
                (KeyCode::Enter, _) => editor.handle_enter(),
                (KeyCode::Backspace, _) => editor.handle_backspace(),
                (KeyCode::Left, _) => editor.move_left(),
                (KeyCode::Right, _) => editor.move_right(),
                (KeyCode::Up, _) => editor.move_up(),
                (KeyCode::Down, _) => editor.move_down(),
                (KeyCode::Char(c), _) if matches!(editor.mode, editor::Mode::Insert) => {
                    match c {
                        '(' | '[' | '{' | '"' | '\'' => editor.auto_close(c),
                        _ => editor.insert_char(c),
                    }
                },
                (KeyCode::Tab, _) => editor.handle_tab(), 
                (_,_) => {}
            }
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}

