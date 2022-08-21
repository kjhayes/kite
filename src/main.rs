#![feature(iter_intersperse)]

mod editor;
mod outputlog;
mod widgets;
mod syntax;

use editor::*;
use syntax::*;

use crossterm::{self, ExecutableCommand};
use crossterm::{event, terminal};
use tui::*;
use tui::layout::*;
use tui::backend::Backend;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(value_parser)]
    file_name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let backend = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = tui::Terminal::new(backend)?;
    let mut stdout = std::io::stdout();

    let mut editor = Editor::new(args.file_name, rust_syntax());

    terminal.hide_cursor()?;
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    let mut running = true;
    while running {
        terminal.draw(|f| ui(f, &mut editor))?;
        
        if event::poll(std::time::Duration::from_millis(10))? {
            let event = event::read()?;
            running = editor.process_event(event)?;
        }
    }

    stdout.execute(terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, editor: &mut Editor) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(2)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),           
        ].as_ref())
        .split(size);

    let paragraph_height: usize = editor.block().inner(chunks[0]).height.into();
    editor.display_height = paragraph_height;
    f.render_widget(editor.widget(), chunks[0]);
    f.render_widget(editor.output.widget(), chunks[1]);
}