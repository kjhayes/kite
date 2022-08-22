use super::Editor;
use super::EditorMode;

use crossterm::event;

impl Editor {
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
                                    let mod_pos = self.cursor_index as u8 % self.num_spaces_per_tab;
                                    for _ in mod_pos..self.num_spaces_per_tab {
                                        self.put_char_on_cursor(' ');
                                        self.move_cursor_right();
                                    }
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
                                        self.remove_grapheme_on_cursor();
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

}