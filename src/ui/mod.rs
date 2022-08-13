use std::io;
use std::time::{Duration, Instant};

use anyhow::Context;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Frame, Terminal,
};

use components::devices::Devices;
use components::library::Library;

use crate::app;

mod components;

pub enum ActiveBlock {
    Library,
    Devices,
}

pub struct Ui {
    app: app::App,
    closed: bool,

    active: ActiveBlock,
    library: Library,
    devices: Devices,
}

impl Ui {
    pub fn new(app: app::App) -> Self {
        let library = Library::new();
        let devices = Devices::new();

        Self {
            app,
            closed: false,
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

        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        self.library.set_list(self.app.load_stations().await?);
        self.devices.set_list(self.app.devices()?);

        loop {
            // TODO: draw on gui on app events?
            //  draw only changed elements?
            terminal.draw(|f| self.draw(f))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                self.handle_event(event::read()?).await?;

                if self.closed {
                    break;
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
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
            ActiveBlock::Library => self.library.render(f, layout[0]),
            ActiveBlock::Devices => self.devices.render(f, layout[0]),
        }

        if let Some(station) = playing {
            use tui::layout::Alignment;
            use tui::text::Spans;
            use tui::widgets::{Block, BorderType, Borders, Paragraph};

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

    async fn handle_event(&mut self, event: Event) -> anyhow::Result<()> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => self.closed = true,
                KeyCode::F(1) => {
                    let stations = match self.app.load_stations().await {
                        Ok(stations) => stations,
                        Err(e) => {
                            log::error!("load stations list failed: {}", e);
                            return Ok(());
                        }
                    };

                    self.active = ActiveBlock::Library;
                    self.library.set_list(stations);
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
                    self.devices.set_list(devices);
                }
                KeyCode::Char('+' | '=') => self.app.volume_up(),
                KeyCode::Char('-') => self.app.volume_down(),
                _ => {}
            };
        }

        match self.active {
            ActiveBlock::Library => self.library.handle_event(event, &mut self.app).await,
            ActiveBlock::Devices => self.devices.handle_event(event, &self.app).await,
        }
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
