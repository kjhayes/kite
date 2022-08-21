use tui::{text::Text, widgets::{Paragraph, Block, Borders}, layout::Alignment};

use crate::widgets::BlockWithInner;



#[derive(Debug)]
pub struct OutputLog {
    pub msg: String,
}
impl OutputLog {
    fn text(&self) -> Text {
        let mut text = Text::default();
        text.extend(Text::from(self.msg.clone()));
        text
    }
    pub fn widget(&self) -> BlockWithInner<Paragraph> {
        let paragraph = Paragraph::new(self.text())
            .alignment(Alignment::Left)
            ;
        let block = Block::default()
            .borders(Borders::ALL)
            .title("log")
            ;
        BlockWithInner {
            block,
            inner: paragraph,
        }
    }
}