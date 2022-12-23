use anyhow::Context;

use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Cell, Row};
use tui::Frame;

use crate::api::Client;
use crate::models::{Station, StationsFilter};
use crate::storage::Storage;

use super::{Component, Styles, Table};

pub struct Library<'a, S: Storage, C: Client> {
    storage: S,
    datasource_table: Table<'a, Datasource<S, C>>,
    datasource_is_active: bool,

    station_table: Table<'a, Station>,
    station_filter: StationsFilter,
}

impl<'a, S: Storage, C: Client> Library<'a, S, C> {
    pub fn new(storage: S, client: C) -> Self
    where
        S: Clone,
    {
        let datasource_table = Table::new(
            vec![
                Datasource::Storage(storage.clone()),
                Datasource::Client(client),
            ],
            |d| Row::new(vec![Cell::from(Span::raw(d.name()))]),
            Styles {
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
            },
        )
        .with_state();

        let station_table = Table::<Station>::new(
            vec![],
            |s| {
                Row::new(vec![
                    Cell::from(Span::raw(format!("🔈 {}", s.name.trim()))),
                    Cell::from(Span::raw(s.country.as_str())),
                    Cell::from(Span::raw(s.codec.as_str())),
                    Cell::from(Span::raw(s.bitrate.to_string())),
                ])
            },
            Styles::default(),
        )
        .with_state();

        Self {
            storage,
            datasource_table,
            datasource_is_active: false,
            station_table,
            station_filter: StationsFilter::default(),
        }
    }

    pub fn handle_up(&mut self) {
        if self.datasource_is_active {
            self.station_table.handle_up();
        } else {
            self.datasource_table.handle_up();
        }
    }

    pub fn handle_down(&mut self) {
        if self.datasource_is_active {
            self.station_table.handle_down();
        } else {
            self.datasource_table.handle_down();
        }
    }

    pub fn handle_left(&mut self) {
        if self.datasource_is_active {
            self.station_table.set_list(vec![]);
            self.datasource_is_active = false;
        }
    }

    pub async fn handle_right(&mut self) -> anyhow::Result<()> {
        if self.datasource_is_active {
            return Ok(());
        }

        if let Some(datasource) = self.datasource_table.get_selected() {
            let stations = datasource.search(&self.station_filter).await?;

            self.station_table.set_list(stations);
            self.datasource_is_active = true;
        }

        Ok(())
    }

    pub async fn handle_save(&mut self) -> anyhow::Result<()> {
        if self.datasource_is_active {
            let station = self.station_table.get_selected().context("not selected")?;

            self.storage.create(station).await?;
        }

        Ok(())
    }

    pub async fn handle_delete(&mut self) -> anyhow::Result<()> {
        if self.datasource_is_active {
            let station = self.station_table.get_selected().context("not selected")?;

            self.storage.delete(station.id).await?;
        }

        Ok(())
    }

    pub fn get_selected(&self) -> Option<&Station> {
        if self.datasource_is_active {
            self.station_table.get_selected()
        } else {
            None
        }
    }

    fn draw_stations<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let rows = self.station_table.build_rows();

        let title = format!(
            "Library [{}]",
            self.datasource_table
                .get_selected()
                .expect("can't be none")
                .name()
        );

        let table = tui::widgets::Table::new(rows)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(title),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .widths(&[
                Constraint::Percentage(60),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
            ]);

        frame.render_stateful_widget(
            table,
            area,
            &mut self.station_table.get_state().expect("state can't be none"),
        );
    }
}

impl<'a, S: Storage, C: Client> Component for Library<'a, S, C> {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        if self.datasource_is_active {
            self.draw_stations(frame, area);
        } else {
            self.datasource_table.draw(frame, area);
        };
    }
}

enum Datasource<S: Storage, C: Client> {
    Storage(S),
    Client(C),
}

impl<S: Storage, C: Client> Datasource<S, C> {
    fn name(&self) -> String {
        match self {
            Datasource::Storage(_) => "📁 storage".to_string(),
            Datasource::Client(c) => format!("🌐 {}", c.name()),
        }
    }

    async fn search(&self, filter: &StationsFilter) -> anyhow::Result<Vec<Station>> {
        match self {
            Datasource::Storage(v) => v.search(filter).await,
            Datasource::Client(v) => v.search(filter).await,
        }
    }
}
