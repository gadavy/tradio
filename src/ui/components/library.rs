use super::Table;
use crate::api::Client;
use crate::models::{Station, StationsFilter};
use crate::storage::Storage;
use crate::ui::components::{Component, Styles};
use std::sync::atomic::{AtomicU32, Ordering};
use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Cell, Row};
use tui::Frame;

#[derive(PartialEq)]
enum Selected {
    Storage,
    Client,
    None,
}

pub struct Library<'a, S, C>
where
    S: Storage,
    C: Client,
{
    storage: S,
    client: C,
    rows: AtomicU32,

    selected: Selected,
    datasource_table: Table<'a, String>,
    storage_table: Table<'a, Station>,
    client_table: Table<'a, Station>,
}

impl<'a, S, C> Library<'a, S, C>
where
    S: Storage,
    C: Client,
{
    pub fn new(storage: S, client: C) -> Self {
        let mut styles = Styles {
            block: Some(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Library"),
            ),
            highlight_style: Some(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            widths: Some(&[Constraint::Percentage(100)]),
        };

        let datasource_table = Table::<String>::new(
            vec!["storage".to_string(), "radio-browser".to_string()],
            |s| Row::new(vec![Cell::from(Span::raw(s))]),
            styles.clone(),
        )
        .with_state();

        styles.block = Some(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Library [storage]"),
        );
        styles.widths = Some(&[
            Constraint::Percentage(60),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ]);

        let storage_table = Table::<Station>::new(
            vec![],
            |s| {
                Row::new(vec![
                    Cell::from(Span::raw(format!("🔈 {}", s.name.trim()))),
                    Cell::from(Span::raw(s.country.as_str())),
                    Cell::from(Span::raw(s.codec.as_str())),
                    Cell::from(Span::raw(s.bitrate.to_string())),
                ])
            },
            styles.clone(),
        )
        .with_state();

        styles.block = Some(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Library [radio-browser]"),
        );

        let client_table = Table::<Station>::new(
            vec![],
            |s| {
                Row::new(vec![
                    Cell::from(Span::raw(format!("🔈 {}", s.name.trim()))),
                    Cell::from(Span::raw(s.country.as_str())),
                    Cell::from(Span::raw(s.codec.as_str())),
                    Cell::from(Span::raw(s.bitrate.to_string())),
                ])
            },
            styles.clone(),
        )
        .with_state();

        Self {
            storage,
            client,
            rows: AtomicU32::default(),
            selected: Selected::None,
            datasource_table,
            storage_table,
            client_table,
        }
    }

    pub fn handle_up(&mut self) {
        match self.selected {
            Selected::Storage => self.storage_table.handle_up(),
            Selected::Client => self.client_table.handle_up(),
            Selected::None => self.datasource_table.handle_up(),
        }
    }

    pub fn handle_down(&mut self) {
        match self.selected {
            Selected::Storage => self.storage_table.handle_down(),
            Selected::Client => self.client_table.handle_down(),
            Selected::None => self.datasource_table.handle_down(),
        }
    }

    pub fn handle_left(&mut self) {
        match self.selected {
            Selected::Storage | Selected::Client => self.selected = Selected::None,
            _ => {}
        }
    }

    pub async fn handle_right(&mut self) -> anyhow::Result<()> {
        if self.selected != Selected::None {
            return Ok(());
        }

        match self.datasource_table.get_selected().map(|s| s.as_str()) {
            Some("storage") => {
                let stations = self.storage.search(&StationsFilter::default()).await?;

                self.storage_table.set_list(stations);
                self.selected = Selected::Storage;
            }
            Some("radio-browser") => {
                let stations = self.client.search(&StationsFilter::default()).await?;

                self.client_table.set_list(stations);
                self.selected = Selected::Client;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn handle_save(&mut self) -> anyhow::Result<()> {
        if self.selected != Selected::Client {
            return Ok(());
        }

        if let Some(station) = self.client_table.get_selected() {
            self.storage.create(station).await?;
        }

        Ok(())
    }

    pub async fn handle_delete(&mut self) -> anyhow::Result<()> {
        if self.selected != Selected::Storage {
            return Ok(());
        }

        if let Some(station) = self.storage_table.get_selected() {
            self.storage.delete(station.id).await?;

            // reload stations list.
            let stations = self.storage.search(&StationsFilter::default()).await?;
            self.storage_table.set_list(stations);
        }

        Ok(())
    }

    pub fn get_selected(&self) -> Option<&Station> {
        match self.selected {
            Selected::Storage => self.storage_table.get_selected(),
            Selected::Client => self.client_table.get_selected(),
            Selected::None => None,
        }
    }
}

impl<'a, S, C> Component for Library<'a, S, C>
where
    S: Storage,
    C: Client,
{
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        match self.selected {
            Selected::Storage => self.storage_table.draw(frame, area),
            Selected::Client => self.client_table.draw(frame, area),
            Selected::None => self.datasource_table.draw(frame, area),
        }
    }

    fn on_resize(&self, _columns: u16, rows: u16) {
        self.rows.store(rows as u32, Ordering::SeqCst)
    }
}
