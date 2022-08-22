use std::{io::Write};

use super::Editor;

use crossterm::{*, style::{Print}};
use syntect::{easy::HighlightLines};
use unicode_segmentation::UnicodeSegmentation;

impl Editor {

    pub fn draw<W>(&mut self, out: &mut W, at: (u16,u16), size: (u16,u16))
        where W: Write
    {
        self.draw_title(out, at, size.0);

        let title_thickness = 1;

        let mut largest_line_number = self.display_top_line_index + self.text_size.1;
        let mut digit_count: usize = 1;
        let line_numbers_thickness = loop {
            largest_line_number /= 10;
            if largest_line_number == 0 {break digit_count;}
            else {digit_count += 1;}
        } + 1;
        
        let size = (size.0 as usize, size.1 as usize);
        self.set_text_size((size.0 - line_numbers_thickness, size.1 - title_thickness));

        self.draw_line_numbers(out, (at.0, at.1 + 1), line_numbers_thickness, digit_count);
        let text_at = (at.0 + line_numbers_thickness as u16, at.1 + title_thickness as u16);
        self.draw_text(out, text_at);

        out.flush().unwrap();
    }

    pub fn draw_title<W>(&self, out: &mut W, at: (u16,u16), length: u16) 
        where W: Write
    {
        let header_msg = &self.header_msg;

        out.queue(style::SetForegroundColor(style::Color::Black)).unwrap();
        out.queue(style::SetBackgroundColor(style::Color::Blue)).unwrap();
        out.queue(cursor::MoveTo(at.0, at.1)).unwrap();
        let mut header = format!("~ {} ~ {}", self.title(), header_msg);
        while header.graphemes(true).count() < length as usize {
            header.push_str(" ");
        }
        out.queue(Print(header)).unwrap();
        out.queue(style::ResetColor).unwrap();
    }

    pub fn draw_line_numbers<W>(&self, out: &mut W, at: (u16,u16), thickness: usize, digit_count: usize) 
        where W: Write 
    {
        let height = self.text_size.1;
        let num_lines = self.lines.iter().count();
        let gap_size = thickness - digit_count;
        for y_offset in 0..height {
            let mut line_number = y_offset + self.display_top_line_index+1;
            for digit_offset in (0..digit_count).rev() {
                let digit = if line_number == 0 || y_offset + self.display_top_line_index >= num_lines {
                    " ".to_string()
                } else {
                    (line_number % 10).to_string()
                };
                
                out.queue(cursor::MoveTo(at.0 + digit_offset as u16, at.1 + y_offset as u16)).unwrap();
                out.queue(Print(digit)).unwrap();

                line_number /= 10;
            }
            for x_offset in 0..gap_size {
                out.queue(cursor::MoveTo(at.0 + (digit_count + x_offset) as u16, at.1 + y_offset as u16)).unwrap();
                out.queue(Print(" ")).unwrap();
            }
        }
    }

    pub fn draw_text<W>(&self, out: &mut W, at: (u16,u16))
        where W: Write
    {
        let syntax = 
            self.syntax_set.find_syntax_for_file(&self.path).unwrap().unwrap_or(
            self.syntax_set.find_syntax_by_extension("txt").unwrap_or_else(
            || {
                eprint!("Critical Error Loading Highlighter");
                std::process::exit(1);
            }
        ));
        
        let default_theme = syntect::highlighting::Theme::default();
        let theme: &syntect::highlighting::Theme = if self.theme_set.themes.contains_key(&self.theme_name) {
                &self.theme_set.themes[&self.theme_name]
            } else {
                &default_theme
            };

        let mut highlight_lines = HighlightLines::new(syntax, &theme);
        
        for (file_line_num, line) in self.lines.iter()
            .chain(std::iter::repeat(&String::from("")))
            .enumerate()
            .take_while(|(file_line_num, _)| *file_line_num < (self.display_top_line_index + self.text_size.1))
        {   
            let mut line = line.clone();
            let grapheme_count = line.graphemes(true).count();
            
            let disp_len = if grapheme_count > self.display_rightmost_index {
                grapheme_count - self.display_rightmost_index
            } else {0};

            if disp_len < self.text_size.0 {
                line.push_str(std::iter::repeat(" ").take(self.text_size.0 - disp_len).collect::<String>().as_str());
            }

            let mut ranges = highlight_lines.highlight_line(&line, &self.syntax_set).unwrap();
            
            if grapheme_count > disp_len {
                (_,ranges) = syntect::util::split_at(&ranges[..], self.display_rightmost_index)    
            }
            if disp_len > self.text_size.0 {
                (ranges,_) = syntect::util::split_at(&ranges[..], self.text_size.0);
            }
            
            if file_line_num >= self.display_top_line_index {
                if self.show_cursor 
                    && file_line_num == self.cursor_line_index 
                    && self.cursor_index >= self.display_rightmost_index
                    && self.cursor_index < self.display_rightmost_index + self.text_size.0 + 1
                {
                    let cursor_char_index = self.cursor_char_index() - self.display_rightmost_char_index(file_line_num);
                    use syntect::highlighting::*;
                    ranges = syntect::util::modify_range(&ranges, cursor_char_index..cursor_char_index+1, 
                        StyleModifier { 
                            foreground: Some(Color::BLACK),
                            background: Some(Color::WHITE),
                            font_style: Some(FontStyle::BOLD)
                        });
                }

                let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], self.use_theme_background);

                let display_line_num = file_line_num - self.display_top_line_index;
                out.queue(cursor::MoveTo(at.0, at.1 + display_line_num as u16)).unwrap();
                out.queue(Print(escaped)).unwrap();
            }
        }
    }
}