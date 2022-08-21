use tui::style::Color;
use synoptic::Highlighter;
use std::collections::HashMap;
use tui::style::Style;

#[derive(Debug)]
pub struct SyntaxSet {
    pub highlighter: Highlighter, 
    pub styles: HashMap<String, Style>
}

impl SyntaxSet {
    pub fn new() -> Self {
        Self {
            highlighter: Highlighter::new(),
            styles: HashMap::new(),
        }
    }

    pub fn add(&mut self, token: &str, regex: &str) {
        self.highlighter.add(regex, &token).unwrap();
    }
    pub fn join(&mut self, token: &str, regex: &[&str]) {
        self.highlighter.join(regex, &token).unwrap();
    }
    pub fn insert(&mut self, token: String, style: Style) {
        self.styles.insert(token, style);
    }
}




pub fn rust_syntax() -> SyntaxSet {
    let mut set = SyntaxSet::new();

    set.join("keyword", &["fn", "return", "pub"]);
    set.join("type", &["bool"]);
    set.join("boolean", &["true", "false"]);
    set.add("comment", r"(?m)(//.*)$");
    set.add("comment", r"(?ms)/\*.*?\*/");
    set.add("string", "\".*?\"");
    set.add("identifier", r"([a-z_][A-Za-z0-9_]*)\s*\(");
    set.add("macro", r"([a-z_][A-Za-z0-9_]*!)\s*");

    set.insert("keyword".to_string(), Style::default().fg(Color::Green));
    set.insert("type".to_string(), Style::default().fg(Color::Red));
    set.insert("comment".to_string(), Style::default().fg(Color::Gray));
    set.insert("string".to_string(), Style::default().fg(Color::Blue));
    set.insert("identifier".to_string(), Style::default().fg(Color::Cyan));
    

    set
}