use crate::api::Client;
use crate::models::Station;
use crate::player::{Device, Player};
use std::sync::Mutex;

#[derive(Default)]
pub struct State {
    device: Option<String>,
    playing: Option<Station>,
}

pub struct App<P: Player> {
    player: P,
    client: Client,

    state: Mutex<State>,
}

impl<P: Player> App<P> {
    pub fn new(player: P, client: Client) -> Self {
        let state = State {
            device: player
                .devices()
                .unwrap() // todo: expect or error.
                .iter()
                .find(|d| d.is_active())
                .map(|d| d.id().to_string()),
            playing: None,
        };

        Self {
            player,
            client,
            state: Mutex::new(state),
        }
    }

    pub async fn play_track(&self, station: &Station) -> anyhow::Result<()> {
        self.player.play(&station.url)?;
        self.state.lock().unwrap().playing = Some(station.clone());

        Ok(())
    }

    pub fn playing(&self) -> Option<Station> {
        self.state.lock().unwrap().playing.as_ref().cloned()
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn play(&self) {
        self.player.resume();
    }

    pub fn stop(&self) {
        self.player.stop();
        self.player.wait_end();
        self.state.lock().unwrap().playing = None;
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

    pub fn devices(&self) -> anyhow::Result<Vec<Device>> {
        self.player.devices()
    }

    pub fn current_device(&self) -> String {
        self.state
            .lock()
            .unwrap()
            .device
            .clone()
            .unwrap_or_default()
    }

    pub async fn use_device(&self, device: &Device) -> anyhow::Result<()> {
        self.player.use_device(device)?;
        self.state.lock().unwrap().device = Some(device.id().to_string());

        let playing = self.state.lock().unwrap().playing.clone();

        // Resume playing current station if exists.
        // TODO: maybe init reader before changing device?
        if let Some(ref station) = playing {
            self.player.play(&station.url)?;
        }

        Ok(())
    }

    pub async fn load_stations(&mut self) -> anyhow::Result<Vec<Station>> {
        self.client.stations().await
    }
}
