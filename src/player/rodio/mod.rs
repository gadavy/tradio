use std::fmt::Formatter;
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
    device: Option<rodio::Device>,
    stream: Option<OutputStream>,
}

impl fmt::Debug for ActiveOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActiveOutput")
            .field("with_device", &self.device.is_some())
            .field("with_stream", &self.stream.is_some())
            .finish()
    }
}

pub struct RodioPlayer {
    sink: Sink,
    queue_rx: SharedSourcesQueue,

    controls: Arc<Controls>,
    active_out: Mutex<ActiveOutput>,
}

impl RodioPlayer {
    const ACCESS_PERIOD: Duration = Duration::from_millis(15);

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

    pub fn default() -> anyhow::Result<Self> {
        let player = Self::new_idle();

        if let Some(device) = player.devices()?.into_iter().find(|d| d.is_default) {
            player.use_device(&device)?
        }

        Ok(player)
    }
}

impl Player for RodioPlayer {
    fn play(&self, track_path: &str) -> anyhow::Result<()> {
        let source = source::SymphoniaSource::from_http(track_path)?;

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

    /// Current volume in percentage [0 - 100].
    fn volume(&self) -> i8 {
        (self.sink.volume() * 100.0).round() as i8
    }

    /// Set volume in percentage [0 - 100].
    fn set_volume(&self, volume: i8) {
        self.sink
            .set_volume(f32::from(volume.clamp(0, 100)) / 100.0);
    }

    fn devices(&self) -> anyhow::Result<Vec<Device>> {
        let host = cpal::default_host();
        let devices = host.output_devices()?;
        let default_device = host.default_output_device();
        let active_out = self.active_out.lock().unwrap();

        let mut result = vec![];

        for device in devices {
            let id = device.name().context("can't get device name")?;
            let is_active = device_equal(&device, &active_out.device)?;
            let is_default = device_equal(&device, &default_device)?;

            result.push(Device {
                id,
                is_active,
                is_default,
            });
        }

        Ok(result)
    }

    fn use_device(&self, device: &Device) -> anyhow::Result<()> {
        let mut devices = cpal::default_host().devices()?;

        let device = loop {
            if let Some(target) = devices.next() {
                if target.name()? == device.id {
                    break target;
                }
            } else {
                return Err(anyhow::Error::msg("device not found"));
            }
        };

        let mut active_out = self.active_out.lock().unwrap();

        if device_equal(&device, &active_out.device)? {
            return Ok(());
        }

        let (stream, handler) = OutputStream::try_from_device(&device)?;

        active_out.stream = Some(stream);
        active_out.device = Some(device);

        handler.play_raw(self.queue_rx.clone())?;

        Ok(())
    }
}

impl fmt::Debug for RodioPlayer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

fn device_equal(a: &rodio::Device, b: &Option<rodio::Device>) -> anyhow::Result<bool> {
    if let Some(b) = b {
        Ok(a.name()? == b.name()?)
    } else {
        Ok(false)
    }
}
