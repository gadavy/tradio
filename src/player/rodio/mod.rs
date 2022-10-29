use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::{fmt, time::Duration};

use anyhow::Context;
use rodio::cpal::traits::HostTrait;
use rodio::queue::SourcesQueueOutput;
use rodio::source::Stoppable;
use rodio::{cpal, DeviceTrait, OutputStream, Sink, Source};

use super::{Device, Player};

mod source;

#[derive(Debug, Default)]
struct Controls {
    stop: AtomicBool,
}

#[derive(Default)]
struct ActiveOutput {
    device: Option<Device>,
    stream: Option<OutputStream>,
}

impl fmt::Debug for ActiveOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActiveOutput")
            .field("device", &self.device)
            .field("with_stream", &self.stream.is_some())
            .finish()
    }
}

// Safety: we guarantee that `OutputStream` cannot be `Send` on Android's AAudio API.
#[cfg(not(target_os = "android"))]
unsafe impl Send for ActiveOutput {}

pub struct Rodio {
    sink: Sink,
    queue_rx: SharedSourcesQueue,

    controls: Arc<Controls>,
    active_out: Mutex<ActiveOutput>,
}

impl Rodio {
    const ACCESS_PERIOD: Duration = Duration::from_millis(15);

    /// Builds new `RodioPlayer` without output stream.
    pub fn new_idle() -> Self {
        let (sink, queue_rx) = Sink::new_idle();
        let queue_rx = SharedSourcesQueue::from(queue_rx);

        Self {
            sink,
            queue_rx,
            controls: Arc::default(),
            active_out: Mutex::default(),
        }
    }

    /// Builds new `RodioPlayer` beginning playback on a default output stream.
    pub fn default() -> anyhow::Result<Self> {
        let player = Self::new_idle();
        let device = player
            .devices()?
            .into_iter()
            .find(|d| d.is_default)
            .context("can't find default device")?;

        player.use_device(&device)?;

        Ok(player)
    }
}

impl Player for Rodio {
    fn play(&self, track_url: &str) -> anyhow::Result<()> {
        let source = source::Symphonia::from_http(track_url)?;

        let controls = self.controls.clone();

        let access = move |src: &mut Stoppable<_>| {
            if controls.stop.load(Ordering::SeqCst) {
                src.stop();
                controls.stop.store(false, Ordering::SeqCst);
            }
        };

        let source = source
            .stoppable()
            .periodic_access(Self::ACCESS_PERIOD, access);

        // clean source queue.
        while self.sink.len() > 0 {
            self.controls.stop.store(true, Ordering::SeqCst);
            self.sink.sleep_until_end();
        }

        self.controls.stop.store(false, Ordering::SeqCst);

        self.sink.append(source);

        Ok(())
    }

    fn wait_end(&self) {
        self.sink.sleep_until_end();
    }

    fn stop(&self) {
        self.controls.stop.store(true, Ordering::SeqCst);
    }

    fn pause(&self) {
        self.sink.pause();
    }

    fn resume(&self) {
        self.sink.play();
    }

    fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    fn volume(&self) -> i8 {
        (self.sink.volume() * 100.0).round() as i8
    }

    fn set_volume(&self, volume: i8) {
        self.sink
            .set_volume(f32::from(volume.clamp(0, 100)) / 100.0);
    }

    fn devices(&self) -> anyhow::Result<Vec<Device>> {
        let host = cpal::default_host();
        let devices = host.output_devices().context("output devices")?;
        let default_device = host.default_output_device();
        let active_out = self.active_out.lock().unwrap();

        let mut result = vec![];

        for device in devices {
            let id = device.name()?;
            let is_active = active_out.device.as_ref().map(Device::id) == Some(&id);

            let is_default = if let Some(ref device) = default_device {
                id == device.name()?
            } else {
                false
            };

            result.push(Device {
                id,
                is_active,
                is_default,
            });
        }

        Ok(result)
    }

    fn use_device(&self, device: &Device) -> anyhow::Result<()> {
        let mut active_out = self.active_out.lock().unwrap();

        if Some(device) == active_out.device.as_ref() {
            return Ok(());
        }

        let mut devices = cpal::default_host().devices()?;

        let rodio_device = loop {
            if let Some(target) = devices.next() {
                if target.name()? == device.id {
                    break target;
                }
            } else {
                return Err(anyhow::Error::msg("device not found"));
            }
        };

        let (stream, handler) = OutputStream::try_from_device(&rodio_device)?;

        active_out.stream = Some(stream);
        active_out.device = Some(device.clone());

        handler.play_raw(self.queue_rx.clone())?;

        Ok(())
    }

    fn active_device(&self) -> Option<Device> {
        self.active_out.lock().unwrap().device.clone()
    }
}

impl fmt::Debug for Rodio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RodioPlayer")
            .field("controls", &self.controls)
            .field("active_out", &self.active_out)
            .finish()
    }
}

#[derive(Clone)]
struct SharedSourcesQueue(Arc<Mutex<SourcesQueueOutput<f32>>>);

impl Source for SharedSourcesQueue {
    fn current_frame_len(&self) -> Option<usize> {
        self.0.lock().unwrap().current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.0.lock().unwrap().channels()
    }

    fn sample_rate(&self) -> u32 {
        self.0.lock().unwrap().sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.0.lock().unwrap().total_duration()
    }
}

impl Iterator for SharedSourcesQueue {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.lock().unwrap().next()
    }
}

impl From<SourcesQueueOutput<f32>> for SharedSourcesQueue {
    fn from(value: SourcesQueueOutput<f32>) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}
