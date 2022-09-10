use std::io;

use anyhow::Context;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures::{FutureExt, StreamExt};
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row};
use tui::{Frame, Terminal};

use components::{Component, Styles, Table};

use crate::app;
use crate::models::Station;
use crate::player::Device;

mod components;

pub enum ActiveBlock {
    Library,
    Devices,
}

pub struct Ui<'a> {
    app: app::App,
    exit: tokio::sync::Notify,

    active: ActiveBlock,
    library: Table<'a, Station>,
    devices: Table<'a, Device>,
}

impl<'a> Ui<'a> {
    pub fn new(app: app::App) -> Self {
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

        Self {
            app,
            exit: tokio::sync::Notify::new(),
            active: ActiveBlock::Library,
            library,
            devices,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        setup_terminal()?;

        let backend = CrosstermBackend::new(io::stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor().context("hide cursor")?;

        self.library.set(self.app.load_stations().await?);
        self.devices.set(self.app.devices()?);

        let mut reader = EventStream::new();

        loop {
            terminal.draw(|f| self.draw(f))?;

            tokio::select! {
                event = reader.next().fuse() => {
                    if let Some(Ok(Event::Key(key_event))) = event {
                        self.handle_key(key_event).await?;
                    }
                },
                _ = self.exit.notified() => break
            }
        }

        self.app.stop();

        shutdown_terminal()
    }

    fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
        let playing = self.app.playing();

        let constraints = if playing.is_some() {
            vec![Constraint::Min(1), Constraint::Length(3)]
        } else {
            vec![Constraint::Min(1)]
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(f.size());

        match self.active {
            ActiveBlock::Library => self.library.draw(f, layout[0]),
            ActiveBlock::Devices => self.devices.draw(f, layout[0]),
        }

        if let Some(station) = playing {
            let title = format!(
                "{:-7} ({} | Volume: {:-2}%)",
                if self.app.is_paused() {
                    "Paused"
                } else {
                    "Playing"
                },
                self.app.current_device_name(),
                self.app.volume()
            );

            let text = vec![Spans::from(format!("Station: {}", station.name.trim()))];

            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .title(title)
                        .borders(Borders::LEFT | Borders::TOP | Borders::RIGHT)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Left);

            f.render_widget(paragraph, layout[1]);
        }
    }

    async fn handle_key(&mut self, event: KeyEvent) -> anyhow::Result<()> {
        match event.code {
            KeyCode::Char('q' | 'й') => self.exit.notify_one(),
            KeyCode::F(1) => {
                let stations = match self.app.load_stations().await {
                    Ok(stations) => stations,
                    Err(e) => {
                        log::error!("load stations list failed: {}", e);
                        return Ok(());
                    }
                };

                self.active = ActiveBlock::Library;
                self.library.set(stations);
            }
            KeyCode::F(2) => {
                let devices = match self.app.devices() {
                    Ok(devices) => devices,
                    Err(e) => {
                        log::error!("load devices list failed: {}", e);
                        return Ok(());
                    }
                };

                self.active = ActiveBlock::Devices;
                self.devices.set(devices);
            }
            KeyCode::Char('+' | '=') => {
                self.app.volume_up();
            }
            KeyCode::Char('-') => {
                self.app.volume_down();
            }
            KeyCode::Up => {
                match self.active {
                    ActiveBlock::Library => self.library.up(),
                    ActiveBlock::Devices => self.devices.up(),
                };
            }
            KeyCode::Down => {
                match self.active {
                    ActiveBlock::Library => self.library.down(),
                    ActiveBlock::Devices => self.devices.down(),
                };
            }
            KeyCode::Enter => {
                match self.active {
                    ActiveBlock::Library => {
                        if let Some(selected) = self.library.selected() {
                            if let Err(e) = self.app.play(selected.clone()) {
                                log::error!("play station {:?} failed {}", selected, e);
                            };
                        }
                    }
                    ActiveBlock::Devices => {
                        if let Some(selected) = self.devices.selected() {
                            if let Err(e) = self.app.use_device(selected) {
                                log::error!("use device {:?} failed {}", selected, e);
                            };
                        }
                    }
                };
            }
            KeyCode::Char('p' | 'з') => {
                if self.app.is_paused() {
                    self.app.resume();
                } else {
                    self.app.pause();
                }
            }
            _ => {}
        }

        Ok(())
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
