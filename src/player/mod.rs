pub use self::rodio::Rodio;

mod rodio;

pub trait Player: Send + Sync {
    /// Starts playing given stream.
    fn play(&self, stream_url: &str) -> anyhow::Result<()>;

    /// Waits until current track ended.
    fn wait_end(&self);

    /// Stops current track. No effect if queue is empty.
    fn stop(&self);

    /// Pauses current track. No effect if already paused.
    fn pause(&self);

    /// Resumes playback of a paused track. No effect if not paused.
    fn resume(&self);

    /// Gets if a player is paused.
    fn is_paused(&self) -> bool;

    /// Current volume in percentage [0 - 100].
    fn volume(&self) -> i8;

    /// Set volume in percentage [0 - 100].
    fn set_volume(&self, volume: i8);

    /// Returns all [Device]s currently available to the system
    /// that support one or more output stream formats.
    fn devices(&self) -> anyhow::Result<Vec<Device>>;

    /// Starts using the given output [Device] for playing tracks.
    fn use_device(&self, device: &Device) -> anyhow::Result<()>;

    /// Return active [Device] if exists.
    fn active_device(&self) -> Option<Device>;
}

#[derive(Debug, Clone)]
pub struct Device {
    id: String,
    is_active: bool,
    is_default: bool,
}

impl Device {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }
}

impl PartialEq<Self> for Device {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
