use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::Style;
use tui::widgets::Block;
use tui::Frame;

pub use table::Table;

pub mod table;

pub trait Component {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, area: Rect);
}

pub struct Styles<'a> {
    pub block: Option<Block<'a>>,
    pub highlight_style: Option<Style>,
    pub widths: Option<&'a [Constraint]>,
}
