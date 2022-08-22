
use syntect::{parsing::SyntaxSet, highlighting::{ThemeSet}};


mod cursor;
mod event;
mod draw;

#[derive(Debug)]
pub enum EditorMode {
    Insert,
}
#[derive(Debug)]
pub struct Editor {

    mode: EditorMode,

    current: bool,
    path: String,
    pub header_msg: String,

    pub extension: String,
    pub theme_name: String,

    lines: Vec<String>,

    text_size: (usize, usize),
    display_top_line_index: usize,
    display_shifted_by_cursor: bool,

    cursor_line_index: usize,
    cursor_index: usize,
    cursor_prefered_index: usize,

    show_cursor: bool,

    use_crlf: bool,

    num_spaces_per_tab: u8,

    pub syntax_set: SyntaxSet,
    pub theme_set: ThemeSet,
    use_theme_background: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            mode: EditorMode::Insert,
            
            current: false,
            path: "Untitled".to_string(),
            header_msg: "".to_string(),

            extension: "".to_string(),
            theme_name: "".to_string(),

            lines: Vec::<String>::new(),
            
            text_size: (0,0),
            display_top_line_index: 0,
            display_shifted_by_cursor: true,
            
            cursor_line_index: 0,
            cursor_index: 0,
            cursor_prefered_index: 0,

            show_cursor: true,

            use_crlf: true,
            num_spaces_per_tab: 4,

            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            use_theme_background: true,
        }
    }
}

impl Editor {
    pub fn new(path: String) -> Self {
        let s = std::fs::read_to_string(&path).unwrap_or(String::from(""));
        let lines = s.lines().map(|s| s.to_owned() ).collect::<Vec<_>>();
        let lines = if !lines.is_empty() {lines} else {vec!["".to_string()]};
        Self {
            path,
            lines,
            ..Default::default()
        }
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

}

