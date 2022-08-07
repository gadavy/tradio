use crate::app;
use crate::models::Station;
use crate::player::Player;
use crossterm::event::{Event, KeyCode};
use tui::widgets::TableState;

pub struct Library {
    list: Vec<Station>,
    selected: TableState,
}

impl Library {
    pub fn new() -> Self {
        Self {
            list: vec![],
            selected: TableState::default(),
        }
    }

    pub fn set_list(&mut self, list: Vec<Station>) {
        self.selected.select(Some(0));
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

    pub fn selected(&self) -> &Station {
        &self.list[self.selected.selected().unwrap_or(0)]
    }

    pub async fn handle_event<P: Player>(
        &mut self,
        event: Event,
        app: &app::App<P>,
    ) -> anyhow::Result<()> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => self.up(),
                KeyCode::Down => self.down(),
                KeyCode::Enter => app.play_track(self.selected()).await?,
                KeyCode::Char('p') => {
                    if app.is_paused() {
                        app.play();
                    } else {
                        app.pause();
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

mod rendering {
    use tui::{
        backend::Backend,
        layout::{Constraint, Rect},
        style::{Color, Modifier, Style},
        text::Span,
        widgets::{Block, BorderType, Borders, Cell, Row, Table},
        Frame,
    };

    use super::Library;

    impl Library {
        pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
            let rows: Vec<Row> = self
                .list
                .iter()
                .map(|s| {
                    Row::new(vec![
                        Cell::from(Span::raw(format!("🔈 {}", s.name.trim()))),
                        Cell::from(Span::raw(s.country.as_str())),
                        Cell::from(Span::raw(s.codec.as_str())),
                        Cell::from(Span::raw(s.bitrate.to_string())),
                    ])
                })
                .collect();

            let table = Table::new(rows)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Library"),
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

            frame.render_stateful_widget(table, area, &mut self.selected);
        }
    }
}
