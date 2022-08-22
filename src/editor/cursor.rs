
use unicode_segmentation::UnicodeSegmentation;



use super::super::Editor;

impl Editor {

    pub (super) fn set_text_size(&mut self, size: (usize, usize)) {
        self.text_size = size;
        if self.display_shifted_by_cursor {
            self.clamp_display_to_cursor();
        }
    }

    pub (super) fn char_index(&self, line_index: usize, grapheme_index: usize) -> Option<usize> {
        let cursor_char = self.line_at_index(line_index)?
            .graphemes(true)
            .take(grapheme_index)
            .fold(0, |acc,val|{acc + val.chars().count()});
        Some(cursor_char)
    }

    pub (super) fn cursor_char_index(&self) -> usize {
        self.char_index(self.cursor_line_index, self.cursor_index).unwrap()
    }

    pub (super) fn display_rightmost_char_index(&self, line_index: usize) -> usize {
        self.char_index(line_index, self.display_rightmost_index).unwrap()
    }

    pub (super) fn collapse_preference(&mut self) {
        self.cursor_prefered_index = self.cursor_index;
    }

    pub (super) fn resolve_cursor_index(&mut self) {
        self.cursor_index = self.cursor_prefered_index;
        let len = self.cursor_current_line().graphemes(true).count();
        if self.cursor_prefered_index > len {
            self.cursor_index = len;
        }
    }

    pub (super) fn line_at_index_mut(&mut self, index: usize) -> Option<&mut String> {
        self.lines.iter_mut().skip(index).next()
    }
    pub (super) fn line_at_index(&self, index: usize) -> Option<&String> {
        self.lines.iter().skip(index).next()
    }
    pub (super) fn cursor_current_line_mut(&mut self) -> &mut String {
        self.line_at_index_mut(self.cursor_line_index).unwrap()
    }
    pub (super) fn cursor_current_line(&self) -> &String {
        self.line_at_index(self.cursor_line_index).unwrap()
    }

    pub (super) fn move_cursor_right(&mut self) {
        if self.cursor_index < self.lines.iter().skip(self.cursor_line_index).next().unwrap().len() {
            self.cursor_index += 1;
            if self.display_shifted_by_cursor {self.clamp_display_to_cursor(); }
            self.collapse_preference();
        } else if self.cursor_line_index < self.lines.iter().count() - 1 {
            self.move_cursor_down();
            self.move_cursor_to_start_of_line();
        }
    }
    pub (super) fn move_cursor_left(&mut self) -> bool {
        if self.cursor_index > 0 {
            self.cursor_index -= 1;
            if self.display_shifted_by_cursor {self.clamp_display_to_cursor(); }
            self.collapse_preference();
            return true;
        } else if self.cursor_line_index > 0 {
            self.move_cursor_up();
            self.move_cursor_to_end_of_line();
            return true;
        } else {
            self.collapse_preference();
            return false;
        }
    }
    pub (super) fn move_cursor_up(&mut self) {
        if self.cursor_line_index > 0 {
            self.cursor_line_index -= 1;
            self.resolve_cursor_index();

            if self.display_shifted_by_cursor {self.clamp_display_to_cursor();}
        }
    }
    pub (super) fn move_cursor_down(&mut self) {
        let len = self.lines.len();
        let max = if len == 0 {0} else {len-1};
        if self.cursor_line_index + 1 <= max {
            self.cursor_line_index += 1;
            self.resolve_cursor_index();

            if self.display_shifted_by_cursor {self.clamp_display_to_cursor();}
        }
    }
    pub (super) fn move_cursor_to_end_of_line(&mut self) {
        let len = self.cursor_current_line().graphemes(true).count();
        self.cursor_index = len;
        self.collapse_preference();
    }
    pub (super) fn move_cursor_to_start_of_line(&mut self) {
        self.cursor_index = 0;
        self.collapse_preference();
    }
    pub (super) fn linesplit_at_cursor(&mut self) {
        let second_half: String;
        {
            let cursor_index = self.cursor_index;
            let current_line = self.cursor_current_line_mut();
            let first_half = String::from(&current_line[0..cursor_index]);
            second_half = String::from(&current_line[cursor_index..]);
            *current_line = first_half;
        }
        self.lines.insert(self.cursor_line_index+1, second_half);
        self.cursor_index = 0;
        self.cursor_line_index += 1;
        self.collapse_preference();
    }

    pub (super) fn put_char_on_cursor(&mut self, c: char) {
        let index = self.cursor_char_index();
        self.cursor_current_line_mut().insert(index, c);
        self.current = false;
        self.collapse_preference();
    }
    pub (super) fn remove_grapheme_on_cursor(&mut self) {
        let cursor_index = self.cursor_index;
        let len_current_line: usize;
        {
            let current_line = self.cursor_current_line_mut();
            len_current_line = current_line.graphemes(true).count();
            if cursor_index < len_current_line {
                *current_line = current_line
                    .graphemes(true)
                    .enumerate()
                    .filter(|(i,_)| *i != cursor_index)
                    .map(|(_,g)|g)
                    .collect::<String>();
                self.current = false;
                self.collapse_preference();
                return;
            }
        } 
        if cursor_index == len_current_line {
            let next_line_content: String;
            if let Some(next_line) = self.line_at_index(self.cursor_line_index+1) {
                next_line_content = next_line.clone();
            } else {
                self.collapse_preference();
                return;
            }
            self.lines.remove(self.cursor_line_index+1);
            self.cursor_current_line_mut().push_str(next_line_content.as_str());
            self.collapse_preference();
            self.current = false;
        }
    }
    pub (super) fn move_display_down(&mut self) {
        self.display_top_line_index += 1;
    }
    pub (super) fn move_display_up(&mut self) {
        self.display_top_line_index = if self.display_top_line_index == 0 {0} else {self.display_top_line_index-1};
    }
    pub (super) fn clamp_display_to_cursor(&mut self) {
        let max_displayed_line_index = self.display_top_line_index + (self.text_size.1 - 1);
        let min_displayed_line_index = self.display_top_line_index;
        if self.cursor_line_index < min_displayed_line_index {
            self.display_top_line_index = self.cursor_line_index;
        } else if self.cursor_line_index > max_displayed_line_index {
            self.display_top_line_index = self.cursor_line_index - (self.text_size.1 - 1);
        }

        if self.cursor_index < self.display_rightmost_index {
            self.display_rightmost_index = self.cursor_index;    
        } else if self.cursor_index > self.display_rightmost_index + (self.text_size.0 - 1) {
            self.display_rightmost_index = self.cursor_index - (self.text_size.0 - 1);
        }
    }

}