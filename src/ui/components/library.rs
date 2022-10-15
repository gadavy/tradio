use super::Table;
use crate::api::Client;
use crate::models::{OrderBy, Station, StationsFilter};
use crate::storage::Storage;
use crate::ui::components::{Component, Styles};
use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Cell, Row};
use tui::Frame;

#[derive(Eq, PartialEq)]
enum Datasource {
    Storage,
    Client,
    None,
}

impl ToString for Datasource {
    fn to_string(&self) -> String {
        match self {
            Datasource::Storage => "📁 storage".to_string(),
            Datasource::Client => "🌐 radio-browser".to_string(),
            Datasource::None => "".to_string(),
        }
    }
}

pub struct Library<'a, S, C>
where
    S: Storage,
    C: Client,
{
    storage: S,
    client: C,

    filter: StationsFilter,
    datasource: Datasource,
    datasource_table: Table<'a, Datasource>,
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

        let mut datasource_table = Table::<Datasource>::new(
            vec![Datasource::Storage, Datasource::Client],
            |s| Row::new(vec![Cell::from(Span::raw(s.to_string()))]),
            styles.clone(),
        )
        .with_state();

        datasource_table.set_selected(Some(0));

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
            filter: StationsFilter {
                order_by: Some(OrderBy::CreatedAt),
                limit: None,
                offset: None,
            },
            datasource: Datasource::None,
            datasource_table,
            storage_table,
            client_table,
        }
    }

    pub fn handle_up(&mut self) {
        match self.datasource {
            Datasource::Storage => self.storage_table.handle_up(),
            Datasource::Client => self.client_table.handle_up(),
            Datasource::None => self.datasource_table.handle_up(),
        }
    }

    pub fn handle_down(&mut self) {
        match self.datasource {
            Datasource::Storage => self.storage_table.handle_down(),
            Datasource::Client => self.client_table.handle_down(),
            Datasource::None => self.datasource_table.handle_down(),
        }
    }

    pub fn handle_left(&mut self) {
        match self.datasource {
            Datasource::Storage | Datasource::Client => self.datasource = Datasource::None,
            _ => {}
        }
    }

    pub async fn handle_right(&mut self) -> anyhow::Result<()> {
        if self.datasource != Datasource::None {
            return Ok(());
        }

        match self.datasource_table.get_selected() {
            Some(Datasource::Storage) => {
                let stations = self.storage.search(&self.filter).await?;

                self.storage_table.set_list(stations);
                self.datasource = Datasource::Storage;
            }
            Some(Datasource::Client) => {
                let stations = self.client.search(&self.filter).await?;

                self.client_table.set_list(stations);
                self.datasource = Datasource::Client;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn handle_save(&mut self) -> anyhow::Result<()> {
        if self.datasource != Datasource::Client {
            return Ok(());
        }

        if let Some(station) = self.client_table.get_selected() {
            self.storage.create(station).await?;
        }

        Ok(())
    }

    pub async fn handle_delete(&mut self) -> anyhow::Result<()> {
        if self.datasource != Datasource::Storage {
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
        match self.datasource {
            Datasource::Storage => self.storage_table.get_selected(),
            Datasource::Client => self.client_table.get_selected(),
            Datasource::None => None,
        }
    }
}

impl<'a, S, C> Component for Library<'a, S, C>
where
    S: Storage,
    C: Client,
{
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        match self.datasource {
            Datasource::Storage => self.storage_table.draw(frame, area),
            Datasource::Client => self.client_table.draw(frame, area),
            Datasource::None => self.datasource_table.draw(frame, area),
        }
    }
}
