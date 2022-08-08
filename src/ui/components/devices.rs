use crossterm::event::{Event, KeyCode};
use tui::widgets::TableState;

use crate::app;
use crate::player::Device;

pub struct Devices {
    list: Vec<Device>,
    selected: TableState,
}

impl Devices {
    pub fn new() -> Self {
        Self {
            list: vec![],
            selected: TableState::default(),
        }
    }

    pub fn set_list(&mut self, list: Vec<Device>) {
        self.selected
            .select(list.iter().position(Device::is_default));
        self.list = list;
    }

    pub fn up(&mut self) {
        let idx = self.selected.selected().unwrap_or(0);

        if idx == 0 {
            self.selected.select(Some(self.list.len() - 1));
        } else {
            self.selected.select(Some(idx - 1));
        }
    }

    pub fn down(&mut self) {
        let idx = self.selected.selected().unwrap_or(0);

        if idx >= self.list.len() - 1 {
            self.selected.select(Some(0));
        } else {
            self.selected.select(Some(idx + 1));
        }
    }

    pub fn selected(&self) -> Option<&Device> {
        if let Some(idx) = self.selected.selected() {
            Some(&self.list[idx])
        } else {
            None
        }
    }

    pub async fn handle_event(
        &mut self,
        event: Event,
        app: &app::App,
    ) -> anyhow::Result<()> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => self.up(),
                KeyCode::Down => self.down(),
                KeyCode::Enter => {
                    if let Some(d) = self.selected() {
                        app.use_device(d).await?;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

mod rendering {
    use tui::style::{Color, Modifier, Style};
    use tui::{
        backend::Backend,
        layout::{Constraint, Rect},
        text::Span,
        widgets::{Block, BorderType, Borders, Cell, Row, Table},
        Frame,
    };

    use super::Devices;

    impl Devices {
        pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
            let rows: Vec<Row> = self
                .list
                .iter()
                .enumerate()
                .map(|(i, d)| {
                    let mut text = (i + 1).to_string();
                    text.push(' ');
                    text.push_str(d.id());

                    if d.is_active() {
                        text.push_str(" (active)");
                    }

                    if d.is_default() {
                        text.push_str(" (default)");
                    }

                    Row::new(vec![Cell::from(Span::raw(text))])
                })
                .collect();

            let table = Table::new(rows)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Devices"),
                )
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .widths(&[Constraint::Percentage(100)]);

            frame.render_stateful_widget(table, area, &mut self.selected);
        }
    }
}
