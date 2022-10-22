use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::Style;
use tui::widgets::Block;
use tui::Frame;

pub use library::Library;
pub use playbar::Playbar;
pub use table::Table;

mod library;
mod playbar;
mod table;

pub trait Component {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, area: Rect);

    fn on_resize(&mut self, _area: Rect) {}
}

#[derive(Default, Clone, Debug)]
pub struct Styles<'a> {
    pub block: Option<Block<'a>>,
    pub highlight_style: Option<Style>,
    pub widths: Option<&'a [Constraint]>,
}
