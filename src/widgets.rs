use tui::{widgets::{Block, Widget}, buffer, layout::Rect};



pub struct BlockWithInner<'a, T: Widget> {
    pub block: Block<'a>,
    pub inner: T,
}
impl<'a, T: Widget> Widget for BlockWithInner<'a, T> {
    fn render(self, area: Rect, buf: &mut buffer::Buffer) {
        let inner = self.block.inner(area);
        self.block.render(area, buf);
        self.inner.render(inner, buf);
    }
}