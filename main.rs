use core::num;
use std::default;
use std::io::Write;

use std::sync::{Arc, Mutex};    
	
use crossterm::style::Stylize;
use crossterm::{self, ExecutableCommand, cursor};
use crossterm::{event, terminal};
use tui::*;
use tui::widgets::*;
use tui::text::*;
use tui::layout::*;
use tui::style::*;
use tui::backend::Backend;

struct BlockWithInner<'a, T: Widget> {
    block: Block<'a>,
    inner: T,
}
impl<'a, T: Widget> Widget for BlockWithInner<'a, T> {
    fn render(self, area: Rect, buf: &mut buffer::Buffer) {
        let inner = self.block.inner(area);
        self.block.render(area, buf);
        self.inner.render(inner, buf);
    }
}

enum EditorMode {
    Insert,
}

struct Editor {
    mode: EditorMode,

    current: bool,
    path: String,

    before_cursor: String,
    after_cursor: String,

    show_cursor: bool,
    cursor_style: Style,

    cr_with_lf: bool,
}

impl Editor {
    fn new(path: String) -> Self {
        let after_cursor = std::fs::read_to_string(&path).unwrap_or(String::new());

        Self {
            mode: EditorMode::Insert,
            current: false,
            path,
            before_cursor: String::new(),
            after_cursor,
            show_cursor: true,
            cursor_style: Style::default().fg(Color::Black).bg(Color::LightCyan),
            cr_with_lf: true,
        }
    }
    
    fn move_cursor_right(&mut self) {
        if !self.after_cursor.is_empty() {
            let c = self.after_cursor.remove(0);
            self.before_cursor.push(c);
            if c == '\r' && self.after_cursor.starts_with('\n') {
                self.move_cursor_right();
            }
        }
    }
    fn move_cursor_left(&mut self) {
        if !self.before_cursor.is_empty() {
            let c = self.before_cursor.pop().unwrap();
            self.after_cursor.insert(0, c);
            if c == '\n' && self.before_cursor.ends_with('\r') {
                self.move_cursor_left();
            }
        }
    }
    fn backspace_before_cursor(&mut self) {
        let popped = self.before_cursor.pop();
        if let Some(c) = popped {
            self.current = false;
            if c == '\n' && self.before_cursor.ends_with('\r') {
                self.backspace_before_cursor();
            }
        }
    }
    fn newline_at_cursor(&mut self) {
        if self.cr_with_lf {
            self.before_cursor.push('\r');
        }
        self.before_cursor.push('\n');
        self.current = false;
    }

    fn process_event(&mut self, event: event::Event) -> Result<bool, Box<dyn std::error::Error>> {
        match self.mode {
            EditorMode::Insert => 
                match event {
                    event::Event::Key(key_event) => {
                        if key_event.modifiers.contains(event::KeyModifiers::CONTROL) {
                            if key_event.code == event::KeyCode::Char('s') {
                                self.save()?;
                            }
                            if key_event.code == event::KeyCode::Char('c') {
                                return Ok(false);
                            }
                        } else {
                            match key_event.code {
                                event::KeyCode::Right => {
                                    self.move_cursor_right();
                                }
                                event::KeyCode::Left => {
                                    self.move_cursor_left();
                                }
                                event::KeyCode::Enter => {
                                    self.newline_at_cursor();
                                }
                                event::KeyCode::Backspace => {
                                    self.backspace_before_cursor();
                                }
                                event::KeyCode::Char(c) => {
                                    self.before_cursor.push(c);
                                    self.current = false;
                                }
                                _ => {}
                            }
                        }
                    }
                
                _ => {}
            }
        }
        Ok(true)
    }

    fn save(&mut self) -> Result<(), std::io::Error> {
        std::fs::write(&self.path, format!("{}{}", self.before_cursor, self.after_cursor).as_bytes())?;
        self.current = true;
        Ok(())
    }

    fn title(&self) -> String {
        let mut ret = if self.current {String::new()} else {"*".to_string()};
        ret.push_str(&self.path);
        ret
    }

    fn text(&self) -> Text {
        let mut tex = Text::default();

        let line_break_before_cursor = self.before_cursor.ends_with('\n') || self.before_cursor.ends_with("\r\n") || self.before_cursor.ends_with("\n\r");

        let num_lines_before_cursor = self.before_cursor.lines().count();

        let num_lines_to_take_before_cursor = if num_lines_before_cursor > 0 {num_lines_before_cursor-1} else {0};
        for line in self.before_cursor.lines().take(num_lines_to_take_before_cursor) {    
            tex.extend(Text::from(if !line.is_empty() {line} else {" "}));
        }
        
        let cursor_line_before = self.before_cursor.lines().rev().next().unwrap_or("");

        let mut after_line_iter = self.after_cursor.lines();
        let cursor_line_after = after_line_iter.next().unwrap_or(" ");
        let cursor_line_after = if cursor_line_after.len() > 0 {cursor_line_after} else {" "};
        
        let cursor_span = Span::styled(&cursor_line_after[0..1], self.cursor_style);
        let cursor_line_after = if cursor_line_after.len() > 1 {&cursor_line_after[1..]} else {""};
        
        if line_break_before_cursor {
            let cursor_span = Span::styled(cursor_span.content, Style::default().bg(Color::Red));
            tex.extend(Text::from(if !cursor_line_before.is_empty() {cursor_line_before} else {" "}));
            tex.extend(Text::from(Spans::from(
                vec![
                    cursor_span,
                    Span::raw(cursor_line_after),
                ]
            )));
        } else {
            let cursor_span = Span::styled(cursor_span.content, Style::default().bg(Color::Blue));
            tex.extend(Text::from(Spans::from(
                vec![
                    Span::raw(cursor_line_before),
                    cursor_span,
                    Span::raw(cursor_line_after),
                ]
            )));
        }

        for line in after_line_iter {
            tex.extend(Text::from(if !line.is_empty() {line} else {" "}));
        }

        tex
    }

    fn widget(&self) -> BlockWithInner<Paragraph> {
        let paragraph = Paragraph::new(self.text())
            .alignment(Alignment::Left)
            ;

        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.title())
            ;

        BlockWithInner {
            block,
            inner: paragraph,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let backend = tui::backend::CrosstermBackend::new(std::io::stdout());
    let mut terminal = tui::Terminal::new(backend)?;
    let mut stdout = std::io::stdout();

    let mut editor = Editor::new("output.txt".to_string());

    terminal.hide_cursor()?;
    terminal::enable_raw_mode()?;
    stdout.execute(terminal::EnterAlternateScreen)?;

    let mut running = true;
    while running {
        terminal.draw(|f| ui(f, &editor))?;
        
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

fn ui<B: Backend>(f: &mut Frame<B>, editor: &Editor) {
    let size = f.size();
    f.render_widget(editor.widget(), size);
}