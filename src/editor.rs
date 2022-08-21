use core::num;
use std::borrow::BorrowMut;
use std::collections::HashMap;

use crate::outputlog::*;
use crate::syntax::SyntaxSet;
use crate::widgets::*;

use crossterm::{self};
use crossterm::{event};
use synoptic::Highlighter;
use synoptic::Token;
use synoptic::highlighter;
use tui::widgets::*;
use tui::text::*;
use tui::layout::*;
use tui::style::*;

#[derive(Debug)]
pub enum EditorMode {
    Insert,
}
#[derive(Debug)]
pub struct Editor {
    pub output: OutputLog,

    mode: EditorMode,

    current: bool,
    path: String,

    lines: Vec<String>,

    display_top_line_index: usize,
    pub display_height: usize,

    display_shifted_by_cursor: bool,

    cursor_line_index: usize,
    cursor_index: usize,

    show_cursor: bool,
    cursor_style: Style,

    use_crlf: bool,

    num_spaces_per_tab: u8,

    syntax_set: SyntaxSet,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            output: OutputLog{msg:"".to_string()},

            mode: EditorMode::Insert,
            current: false,
            path: "Untitled".to_string(),
            lines: Vec::<String>::new(),
            display_top_line_index: 0,
            display_height: 1,
            display_shifted_by_cursor: true,
            cursor_line_index: 0,
            cursor_index: 0,
            show_cursor: true,
            cursor_style: Style::default().fg(Color::Black).bg(Color::White),
            use_crlf: true,
            num_spaces_per_tab: 4,
            syntax_set: SyntaxSet::new(),
        }
    }
}

impl Editor {
    pub fn new(path: String, syntax: SyntaxSet) -> Self {
        let s = std::fs::read_to_string(&path).unwrap_or(String::from(""));
        let lines = s.lines().map(|s| s.to_owned() ).collect::<Vec<_>>();
        let lines = if !lines.is_empty() {lines} else {vec!["".to_string()]};
        Self {
            path,
            lines,
            syntax_set: syntax,
            ..Default::default()
        }
    }

    fn resolve_cursor_index(&mut self) {
        let len = self.cursor_current_line().len();
        if self.cursor_index > len {
            self.cursor_index = len;
        }
    }

    fn line_at_index(&mut self, index: usize) -> Option<&mut String> {
        self.lines.iter_mut().skip(index).next()
    }
    fn cursor_current_line(&mut self) -> &mut String {
        self.line_at_index(self.cursor_line_index).unwrap()
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_index < self.lines.iter().skip(self.cursor_line_index).next().unwrap().len() {
            self.cursor_index += 1;
        } else if self.cursor_line_index < self.lines.iter().count() - 1 {
            self.move_cursor_down();
            self.move_cursor_to_start_of_line();
        }
    }
    fn move_cursor_left(&mut self) -> bool {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
            return true;
        } else if self.cursor_line_index > 0 {
            self.move_cursor_up();
            self.move_cursor_to_end_of_line();
            return true;
        } else {
            return false;
        }
    }
    fn move_cursor_up(&mut self) {
        if self.cursor_line_index > 0 {
            self.cursor_line_index -= 1;
            self.resolve_cursor_index();

            if self.display_shifted_by_cursor {self.clamp_display_to_cursor();}
        }
    }
    fn move_cursor_down(&mut self) {
        let len = self.lines.len();
        let max = if len == 0 {0} else {len-1};
        if self.cursor_line_index + 1 <= max {
            self.cursor_line_index += 1;
            self.resolve_cursor_index();

            if self.display_shifted_by_cursor {self.clamp_display_to_cursor();}
        }
    }
    fn move_cursor_to_end_of_line(&mut self) {
        let len = self.cursor_current_line().len();
        self.cursor_index = len;
    }
    fn move_cursor_to_start_of_line(&mut self) {
        self.cursor_index = 0;
    }
    fn linesplit_at_cursor(&mut self) {
        let second_half: String;
        {
            let cursor_index = self.cursor_index;
            let current_line = self.cursor_current_line();
            let first_half = String::from(&current_line[0..cursor_index]);
            second_half = String::from(&current_line[cursor_index..]);
            *current_line = first_half;
        }
        self.lines.insert(self.cursor_line_index+1, second_half);
        self.cursor_index = 0;
        self.cursor_line_index += 1;
    }
    fn put_char_on_cursor(&mut self, c: char) {
        let cursor_index = self.cursor_index;
        self.cursor_current_line().insert(cursor_index, c);
    }
    fn remove_char_on_cursor(&mut self) {
        let cursor_index = self.cursor_index;
        let len_current_line: usize;
        {
            let current_line = self.cursor_current_line();
            len_current_line = current_line.len();
            if cursor_index < len_current_line {
                current_line.remove(cursor_index);
                return;
            }
        } 
        if cursor_index == len_current_line {
            let next_line_content: String;
            if let Some(next_line) = self.line_at_index(self.cursor_line_index+1) {
                next_line_content = next_line.clone();
            } else {
                return;
            }
            self.lines.remove(self.cursor_line_index+1);
            self.cursor_current_line().push_str(next_line_content.as_str());
        }
    }
    fn move_display_down(&mut self) {
        self.display_top_line_index += 1;
    }
    fn move_display_up(&mut self) {
        self.display_top_line_index = if self.display_top_line_index == 0 {0} else {self.display_top_line_index-1};
    }
    fn clamp_display_to_cursor(&mut self) {
        let max_displayed_line_index = self.display_top_line_index + (self.display_height - 1);
        let min_displayed_line_index = self.display_top_line_index;
        if self.cursor_line_index < min_displayed_line_index {
            self.display_top_line_index = self.cursor_line_index;
        } else if self.cursor_line_index > max_displayed_line_index {
            self.display_top_line_index = self.cursor_line_index - (self.display_height - 1);
        }
    }

    pub fn process_event(&mut self, event: event::Event) -> Result<bool, Box<dyn std::error::Error>> {
        match self.mode {
            EditorMode::Insert => 
                match event {
                    event::Event::Key(key_event) => {
                        if key_event.modifiers.contains(event::KeyModifiers::CONTROL) {
                            match key_event.code {
                                event::KeyCode::Char('c') => {
                                    return Ok(false);
                                }
                                event::KeyCode::Char('s') => {
                                    self.save()?;
                                }
                                event::KeyCode::Right => {
                                    self.move_cursor_to_end_of_line();
                                }
                                event::KeyCode::Left => {
                                    self.move_cursor_to_start_of_line();
                                }
                                event::KeyCode::Down => {
                                    self.move_display_down();
                                }
                                event::KeyCode::Up => {
                                    self.move_display_up();
                                }

                                _ => {}
                            }
                        } else {
                            match key_event.code {
                                event::KeyCode::Tab => {
                                    self.put_char_on_cursor('\t');
                                    self.move_cursor_right();
                                }
                                event::KeyCode::Right => {
                                    self.move_cursor_right();
                                }
                                event::KeyCode::Left => {
                                    self.move_cursor_left();
                                }
                                event::KeyCode::Up => {
                                    self.move_cursor_up();
                                }
                                event::KeyCode::Down => {
                                    self.move_cursor_down();
                                }
                                event::KeyCode::Enter => {
                                    self.linesplit_at_cursor();
                                }
                                event::KeyCode::Backspace => {
                                    if self.move_cursor_left() {
                                        self.remove_char_on_cursor();
                                    }
                                }
                                event::KeyCode::Char(c) => {
                                    self.put_char_on_cursor(c);
                                    self.move_cursor_right();
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
        let content = self.lines.iter().map(|l|l.clone()).intersperse(if self.use_crlf {"\r\n"} else {"\n"}.to_string()).collect::<String>();
        std::fs::write(&self.path, content.as_bytes())?;
        self.current = true;
        Ok(())
    }

    fn title(&self) -> String {
        let mut ret = if self.current {String::new()} else {"*".to_string()};
        ret.push_str(&self.path);
        ret
    }

    fn text(&self, top_line_index: usize, num_lines: usize) -> Text {
        let mut tex = Text::default();

        let mut curr_kind: String = String::new();
        let total_file = self.lines.iter().map(|s|s.to_string()).intersperse("\n".to_string()).collect::<String>();
        let tok_rows = self.syntax_set.highlighter.run(&total_file);
        for row in tok_rows {
            let mut spans = Vec::<Span>::new();
        
            for (index, tok) in row.into_iter().enumerate() {
                match tok {
                    Token::Start(kind) => {
                        curr_kind = kind;
                    }
                    Token::Text(text) => {
                        let opt = self.syntax_set.styles.get(&curr_kind);
                        if let Some(style) = opt {
                            spans.push(Span::styled(text, *style))
                        } else {
                            spans.push(Span::raw(text));
                        }
                    }
                    Token::End(_) => {
                        curr_kind = String::new();
                    }
                }
            }
            
            tex.extend(Text::from(Spans::from(spans)));
        }

        tex
    }

    fn paragraph(&self) -> Paragraph {
        Paragraph::new(self.text(self.display_top_line_index, self.display_height))
            .alignment(Alignment::Left)
    }

    pub fn block(&self) -> Block {
        Block::default()
            .borders(Borders::ALL)
            .title(self.title())
    }

    pub fn widget(&self) -> BlockWithInner<Paragraph> {
        BlockWithInner {
            block: self.block(),
            inner: self.paragraph(),
        }
    }
}

