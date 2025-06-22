mod editor;

use std::env;
use std::io;
use std::path::Path;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal;
use editor::read_file;
use editor::Editor;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    let file_path = if args.len() < 2 {
        ".".to_string()
    }else {
        args[1].clone()
    };

    if args.len() == 2 && (args[1] == "-h" || args[1] == "--help") {
        command_list();
        return Ok(());
    }

    let mut editor = Editor::new(&file_path);

    if file_path != "." { editor.mode = editor::Mode::Insert };

    terminal::enable_raw_mode()?;

    let _raw = terminal::enable_raw_mode();

    loop {
        editor.render();
        editor.draw_cursor();
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            match (code, modifiers) {
                (KeyCode::Esc, _) => {
                    editor.mode = editor::Mode::Command;
                },
                (KeyCode::Char('i'), _) if matches!(editor.mode, editor::Mode::Command) && editor.command.is_empty() => {
                    editor.mode = editor::Mode::Insert;
                },
                (KeyCode::Enter, _) if matches!(editor.mode, editor::Mode::Command) => {
                    if editor.command.starts_with(":e ") {
                        let path_arg = editor.command[2..].trim().to_string();
                        editor.open_file_from_command(&path_arg);
                    } else if editor.command.starts_with(":w ") {
                        let path_arg = editor.command[2..].trim().to_string();
                        file_path = path_arg;
                        editor.save()?;
                        editor.status_message = "File Saved".to_string();
                    } else if editor.command == ":w" {
                        editor.save()?;
                        editor.status_message = "File Saved".to_string();
                    } else if editor.command == ":q" {
                        break;
                    } else if editor.command == ":wq" {
                        editor.save()?;
                        break;
                    } else {
                        editor.status_message = "Unknow command".to_string();
                    }
                
                    editor.command.clear();
                },
                (KeyCode::Backspace, _) if matches!(editor.mode, editor::Mode::Command) => {
                    editor.command.pop();
                },
                (KeyCode::Char(c), _) if matches!(editor.mode, editor::Mode::Command) => {
                    editor.command.push(c);
                },
                (KeyCode::Up, _) if matches!(editor.mode, editor::Mode::Command) => {
                    editor.move_up_files();
                },
                (KeyCode::Down, _) if matches!(editor.mode, editor::Mode::Command) => {
                    editor.move_down_files();
                },
                (KeyCode::Right, _) if matches!(editor.mode, editor::Mode::Command) => {
                   editor.open_selected(); 
                },
                (KeyCode::Left, _) if matches!(editor.mode, editor::Mode::Command) => {
                   editor.go_back(); 
                },
                (KeyCode::Enter, _) => editor.handle_enter(),
                (KeyCode::Backspace, _) => editor.handle_backspace(),
                (KeyCode::Delete, _) => editor.handle_delete(),
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

fn command_list() {
    println!("++=====================================================================++");
    println!("||                    ReEdit - Terminal Text Editor                    ||");
    println!("||                                                                     ||");
    println!("|| Usage:                                                              ||");
    println!("||    reedit <file_path>        - Open file                            ||");
    println!("||    reedit                    - Open empty file in current directory ||");
    println!("||    reedit -h | reedit --help - Show this message                    ||");
    println!("||                                                                     ||");
    println!("|| Keyboard Commands:                                                  ||");
    println!("||    Esc                       - Enter command mode                   ||");
    println!("||    i                         - Enter insert mode                    ||");
    println!("||    :w                        - Save File                            ||");
    println!("||    :q                        - Quit                                 ||");
    println!("||    :wq                       - Save and quit                        ||");
    println!("||    :e <file>                 - Edit new file                        ||");
    println!("||    arrows (Insert Mode)      - Navigate in file                     ||");
    println!("||    arrows (Command Mode)     - Browse files                         ||");
    println!("++=====================================================================++");
}
