use tui::backend::Backend;
use tui::layout::{Alignment, Rect};
use tui::text::Spans;
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;

use crate::models::Station;
use crate::player::{Device, Player};

use super::Component;

pub struct Playbar {
    is_paused: bool,
    volume: i8,
    device: String,
    station: Option<String>,
}

impl Playbar {
    pub fn new<P: Player>(player: &P) -> Self {
        Self {
            is_paused: player.is_paused(),
            volume: player.volume(),
            device: Self::device_name(player),
            station: None,
        }
    }

    pub fn set_player_settings<P: Player>(&mut self, player: &P) {
        self.is_paused = player.is_paused();
        self.volume = player.volume();
        self.device = Self::device_name(player);
    }

    pub fn set_station(&mut self, station: Option<&Station>) {
        self.station = station.map(|s| s.name.trim().to_string());
    }

    fn get_title(&self) -> String {
        format!(
            "{:-7} ({} | Volume: {:-2}%)",
            if self.is_paused || self.station.is_none() {
                "Paused"
            } else {
                "Playing"
            },
            self.device,
            self.volume
        )
    }

    fn get_text(&self) -> Vec<Spans> {
        self.station.as_ref().map_or_else(Vec::new, |station| {
            vec![Spans::from(format!("Station: {}", station.trim()))]
        })
    }

    fn device_name<P: Player>(player: &P) -> String {
        player
            .active_device()
            .as_ref()
            .map_or("NONE", Device::id)
            .to_string()
    }
}

impl Component for Playbar {
    fn draw<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let paragraph = Paragraph::new(self.get_text())
            .block(
                Block::default()
                    .title(self.get_title())
                    .borders(Borders::LEFT | Borders::TOP | Borders::RIGHT)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }
}
