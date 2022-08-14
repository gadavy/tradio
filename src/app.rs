use crate::api::Client;
use crate::models::Station;
use crate::player::{Device, Player};
use crate::storage::Storage;

pub struct App {
    pub player: Box<dyn Player>,
    pub storage: Box<dyn Storage>,
    pub client: Box<dyn Client>,

    pub active_device: Option<Device>,
    pub playing_station: Option<Station>,
}

impl App {
    pub fn new<P, S, C>(player: P, storage: S, client: C) -> anyhow::Result<Self>
    where
        P: Player + 'static,
        S: Storage + 'static,
        C: Client + 'static,
    {
        let active_device = player.devices()?.into_iter().find(Device::is_active);

        Ok(Self {
            player: Box::new(player),
            storage: Box::new(storage),
            client: Box::new(client),
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
        self.playing_station = None;
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
        self.active_device
            .as_ref()
            .map_or("NONE", Device::id)
            .to_string()
    }

    pub fn devices(&self) -> anyhow::Result<Vec<Device>> {
        self.player.devices()
    }

    pub fn use_device(&self, device: &Device) -> anyhow::Result<()> {
        self.player.use_device(device)?;

        Ok(())
    }

    pub async fn load_stations(&mut self) -> anyhow::Result<Vec<Station>> {
        self.client.stations().await
    }
}
