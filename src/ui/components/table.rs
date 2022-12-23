use std::sync::Arc;

use tui::backend::Backend;
use tui::layout::Rect;
use tui::widgets::{Row, TableState};
use tui::Frame;

use super::{Component, Styles};

pub struct Table<'a, T> {
    list: Vec<T>,
    state: Option<TableState>,

    row_builder: Arc<dyn Fn(&T) -> Row>,
    styles: Styles<'a>,
}

impl<'a, T> Table<'a, T> {
    pub fn new<RB>(list: Vec<T>, row_builder: RB, styles: Styles<'a>) -> Self
    where
        RB: Fn(&T) -> Row + 'static,
    {
        Self {
            list,
            state: None,
            row_builder: Arc::new(row_builder),
            styles,
        }
    }

    pub fn with_state(mut self) -> Self {
        let mut state = TableState::default();
        state.select(Some(0));

        self.state = Some(state);

        self
    }

    pub fn set_list(&mut self, list: Vec<T>) {
        self.list = list;

        if self.state.is_some() && !self.list.is_empty() {
            self.state.as_mut().unwrap().select(Some(0));
        }
    }

    pub fn handle_up(&mut self) {
        if let Some(ref mut state) = self.state {
            let idx = state.selected().unwrap_or(0);

            if idx == 0 {
                state.select(Some(self.list.len() - 1));
            } else {
                state.select(Some(idx - 1));
            }
        }
    }

    pub fn handle_down(&mut self) {
        if let Some(ref mut state) = self.state {
            let idx = state.selected().unwrap_or(0);

            if idx >= self.list.len() - 1 {
                state.select(Some(0));
            } else {
                state.select(Some(idx + 1));
            }
        }
    }

    pub fn get_selected(&self) -> Option<&T> {
        if let Some(ref state) = self.state {
            Some(&self.list[state.selected().unwrap_or(0)])
        } else {
            None
        }
    }

    pub fn get_state(&self) -> Option<TableState> {
        self.state.clone()
    }

    pub(super) fn build_rows(&self) -> Vec<Row> {
        self.list.iter().map(|s| (self.row_builder)(s)).collect()
    }
}

impl<'a, T> Component for Table<'a, T> {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let rows: Vec<Row> = self.build_rows();
        let mut table = tui::widgets::Table::new(rows);

        if let Some(ref block) = self.styles.block {
            table = table.block(block.clone());
        }

        if let Some(highlight_style) = self.styles.highlight_style {
            table = table.highlight_style(highlight_style);
        }

        if let Some(widths) = self.styles.widths {
            table = table.widths(widths);
        }

        if let Some(ref state) = self.state {
            frame.render_stateful_widget(table, area, &mut state.clone());
        } else {
            frame.render_widget(table, area);
        }
    }
}
