use crate::api::Client;
use crate::models::Station;
use crate::player::{Device, Player};

pub struct App {
    pub player: Box<dyn Player>,
    pub client: Client,

    pub active_device: Option<Device>,
    pub playing_station: Option<Station>,
}

impl App {
    pub fn new<P: Player + 'static>(player: P, client: Client) -> anyhow::Result<Self> {
        let active_device = player.devices()?.into_iter().find(|d| d.is_active());

        Ok(Self {
            player: Box::new(player),
            client,
            active_device,
            playing_station: None,
        })
    }

    pub fn play(&mut self, station: Station) -> anyhow::Result<()> {
        self.player.play(&station.url)?;
        self.playing_station = Some(station);

        Ok(())
    }

    pub fn playing(&self) -> Option<&Station> {
        self.playing_station.as_ref()
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.resume();
    }

    pub fn stop(&mut self) {
        self.player.stop();
        self.playing_station = None
    }

    pub fn volume(&self) -> i8 {
        self.player.volume()
    }

    pub fn volume_up(&self) {
        self.player.set_volume(self.player.volume() + 5);
    }

    pub fn volume_down(&self) {
        self.player.set_volume(self.player.volume() - 5);
    }

    pub fn current_device_name(&self) -> String {
        self.active_device.as_ref().map(|d| d.id()).unwrap_or("NONE").to_string()
    }

    pub fn devices(&self) -> anyhow::Result<Vec<Device>> {
        self.player.devices()
    }

    pub async fn use_device(&self, device: &Device) -> anyhow::Result<()> {
        self.player.use_device(device)?;

        Ok(())
    }

    pub async fn load_stations(&mut self) -> anyhow::Result<Vec<Station>> {
        self.client.stations().await
    }
}
