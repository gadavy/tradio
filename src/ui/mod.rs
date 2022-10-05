use std::io;

use anyhow::Context;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::{FutureExt, StreamExt};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Cell, Row};
use tui::{Frame, Terminal};

use components::{Component, Playbar, Styles, Table};

use crate::api::Client;
use crate::models::Station;
use crate::player::{Device, Player};
use crate::storage::Storage;

mod components;

#[derive(PartialEq)]
pub enum ActiveLayout {
    Library,
    Devices,
}

pub struct Ui<'a, P, S, C>
where
    P: Player,
    S: Storage,
    C: Client,
{
    player: P,
    storage: S,
    client: C,

    active_layout: ActiveLayout,

    library: Table<'a, Station>,
    devices: Table<'a, Device>,
    playbar: Playbar,
}

impl<'a, P, S, C> Ui<'a, P, S, C>
where
    P: Player,
    S: Storage,
    C: Client,
{
    pub fn new(player: P, storage: S, client: C) -> Self {
        let library = Table::<Station>::new(
            vec![],
            |s| {
                Row::new(vec![
                    Cell::from(Span::raw(format!("🔈 {}", s.name.trim()))),
                    Cell::from(Span::raw(s.country.as_str())),
                    Cell::from(Span::raw(s.codec.as_str())),
                    Cell::from(Span::raw(s.bitrate.to_string())),
                ])
            },
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
                widths: Some(&[
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                ]),
            },
        )
        .with_state();

        let devices = Table::<Device>::new(
            vec![],
            |d| {
                let mut text = d.id().to_string();

                if d.is_active() {
                    text.push_str(" (active)");
                }

                if d.is_default() {
                    text.push_str(" (default)");
                }

                Row::new(vec![Cell::from(Span::raw(text))])
            },
            Styles {
                block: Some(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Devices"),
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

        let playbar = Playbar::new(&player);

        Self {
            player,
            storage,
            client,
            active_layout: ActiveLayout::Library,
            library,
            devices,
            playbar,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        setup_terminal()?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor().context("hide cursor")?;

        self.library.set(self.client.stations().await?);
        self.devices.set(self.player.devices()?);

        let mut reader = EventStream::new();

        loop {
            terminal.draw(|f| self.draw(f))?;

            tokio::select! {
                event = reader.next().fuse() => {
                    let key_event = match event {
                        Some(Ok(Event::Key(key_event))) => key_event,
                        _ => continue
                    };

                    match self.handle_key(key_event).await {
                        Ok(false) => break,
                        Ok(true) => continue,
                        Err(e) => log::error!("handle key {:?}: {:?}", key_event.code, e),
                    }
                },
            }
        }

        self.player.stop();
        self.playbar.set_station(None);

        shutdown_terminal()
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let constraints = vec![Constraint::Min(1), Constraint::Length(3)];

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(f.size());

        match self.active_layout {
            ActiveLayout::Library => self.library.draw(f, layout[0]),
            ActiveLayout::Devices => self.devices.draw(f, layout[0]),
        }

        self.playbar.draw(f, layout[1])
    }

    async fn handle_key(&mut self, event: KeyEvent) -> anyhow::Result<bool> {
        match event.code {
            KeyCode::Char('q' | 'й') => return Ok(false),
            KeyCode::F(1) => self.handle_set_layout(ActiveLayout::Library)?,
            KeyCode::F(2) => self.handle_set_layout(ActiveLayout::Devices)?,
            KeyCode::F(5) => self.handle_refresh().await?,
            KeyCode::Char('+' | '=') => self.player.set_volume(self.player.volume() + 5),
            KeyCode::Char('-') => self.player.set_volume(self.player.volume() - 5),
            KeyCode::Up => self.handle_up(),
            KeyCode::Down => self.handle_down(),
            KeyCode::Enter => self.handle_enter()?,
            KeyCode::Char('p' | 'з') => self.handle_pause(),
            _ => {}
        }

        self.playbar.set_player_settings(&self.player);

        Ok(true)
    }

    fn handle_set_layout(&mut self, layout: ActiveLayout) -> anyhow::Result<()> {
        if layout == ActiveLayout::Devices {
            self.devices.set(self.player.devices()?)
        }

        self.active_layout = layout;

        Ok(())
    }

    async fn handle_refresh(&mut self) -> anyhow::Result<()> {
        match self.active_layout {
            ActiveLayout::Library => self.library.set(self.client.stations().await?),
            ActiveLayout::Devices => self.devices.set(self.player.devices()?),
        }

        Ok(())
    }

    fn handle_enter(&mut self) -> anyhow::Result<()> {
        match self.active_layout {
            ActiveLayout::Library => {
                if let Some(selected) = self.library.selected() {
                    self.player.play(&selected.url)?;
                    self.playbar.set_station(Some(selected));
                }
            }
            ActiveLayout::Devices => {
                if let Some(selected) = self.devices.selected() {
                    self.player.use_device(selected)?;
                }
            }
        };

        Ok(())
    }

    fn handle_pause(&mut self) {
        if self.player.is_paused() {
            self.player.resume();
        } else {
            self.player.pause();
        }
    }

    fn handle_up(&mut self) {
        match self.active_layout {
            ActiveLayout::Library => self.library.up(),
            ActiveLayout::Devices => self.devices.up(),
        };
    }

    fn handle_down(&mut self) {
        match self.active_layout {
            ActiveLayout::Library => self.library.down(),
            ActiveLayout::Devices => self.devices.down(),
        };
    }
}

fn setup_terminal() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("execute")?;
    enable_raw_mode().context("enable raw mod")?;

    std::panic::set_hook(Box::new(|info| {
        shutdown_terminal().expect("can't graceful shutdown terminal");
        eprintln!("{:?}", info);
    }));

    Ok(())
}

fn shutdown_terminal() -> anyhow::Result<()> {
    disable_raw_mode().context("disable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen).context("execute")
}
