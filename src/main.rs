#![feature(iter_intersperse)]

mod editor;
use editor::*;

use std::sync::{Arc, Mutex};

use crossterm::{self, ExecutableCommand};
use crossterm::{event, terminal};

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    file_name: String,

    #[clap(value_parser)]
    theme: Option<String>,

    #[clap(value_parser)]
    extra_themes_folder: Option<String>,
}

enum RenderThreadMsg {
    Halt,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut editor = Editor::new(args.file_name.clone());
    editor.extension = args.file_name.chars().rev().take_while(|c| *c != '.').collect::<Vec<_>>().into_iter().rev().collect();
    editor.theme_name = if let Some(theme) = args.theme {theme.clone()} else {"Solarized (dark)".to_string()};
    if let Some(theme_folder) = args.extra_themes_folder {editor.theme_set.add_from_folder(theme_folder).unwrap();}

    let editor = Mutex::new(editor);
    let editor = Arc::new(editor);

    let (tx, rx) = std::sync::mpsc::channel();

    let render_thread;
    {
        let editor = editor.clone();
        render_thread = std::thread::spawn(
            move || -> Result<(), std::io::Error> {
                let mut stdout = std::io::stdout();

                terminal::enable_raw_mode()?;
                stdout.execute(terminal::EnterAlternateScreen)?;
                stdout.execute(crossterm::cursor::Hide).unwrap();

                'renderloop: loop {
                    if let Ok(msg) = rx.try_recv() {
                        match msg {
                            RenderThreadMsg::Halt => {break 'renderloop;}
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    editor.lock().unwrap().draw(&mut stdout, (0,0), terminal::size().unwrap());
                }

                stdout.execute(crossterm::cursor::Show).unwrap();
                stdout.execute(terminal::LeaveAlternateScreen)?;
                terminal::disable_raw_mode()?;
                
                Ok(())
            }
        );
    }

    let mut running = true;
    while running {
        
        if event::poll(std::time::Duration::from_millis(100))? {
            let event = event::read()?;
            running = editor.lock().unwrap().process_event(event)?;
        }
    }

    tx.send(RenderThreadMsg::Halt).unwrap();
    render_thread.join().unwrap().unwrap();

    Ok(())
}
