use std::fmt::Formatter;
use std::time::Duration;
use std::{fmt, io};

use anyhow::Context;
use rodio::Source;
use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions, ReadOnlySource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};

pub struct Symphonia {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,

    offset: usize,
    buffer: SampleBuffer<i16>,
    spec: SignalSpec,
}

impl Symphonia {
    pub fn from_http(url: &str) -> anyhow::Result<Self> {
        let resp = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .build()?
            .get(url)
            .send()
            .context("get http response")?;

        Self::from_reader(resp)
    }

    pub fn from_reader<R>(reader: R) -> anyhow::Result<Self>
    where
        R: io::Read + Send + Sync + 'static,
    {
        let rs = ReadOnlySource::new(reader);
        let mss = MediaSourceStream::new(Box::new(rs), MediaSourceStreamOptions::default());

        let probe = get_probe().format(
            &Hint::new(),
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?;

        let mut reader = probe.format;
        let track = reader.default_track().context("track must by found")?;
        let mut decoder = get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

        let packet = reader.next_packet().context("packet must by found")?;
        let decoded_buf = decoder.decode(&packet).context("decode packet")?;
        let spec = *decoded_buf.spec();

        let mut buffer = SampleBuffer::new(decoded_buf.capacity() as u64, spec);
        buffer.copy_interleaved_ref(decoded_buf);

        Ok(Self {
            reader,
            decoder,
            offset: 0,
            buffer,
            spec,
        })
    }
}

impl Source for Symphonia {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buffer.samples().len())
    }

    fn channels(&self) -> u16 {
        self.spec
            .channels
            .count()
            .try_into()
            .expect("unexpected u16 overflow")
    }

    fn sample_rate(&self) -> u32 {
        self.spec.rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for Symphonia {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.buffer.len() {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(_) => return None,
            };

            let decoded = match self.decoder.decode(&packet) {
                Ok(buffer) => buffer,
                Err(_) => return None,
            };

            let mut buffer = SampleBuffer::new(decoded.capacity() as u64, *decoded.spec());
            buffer.copy_interleaved_ref(decoded);

            self.buffer = buffer;
            self.offset = 0;
        }

        let sample = self.buffer.samples()[self.offset];
        self.offset += 1;

        Some(sample)
    }
}

impl fmt::Debug for Symphonia {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut formatter = f.debug_struct("SymphoniaSource");
        formatter.field("offset", &self.offset);
        formatter.field("buffer", &self.buffer.len());
        formatter.field("spec", &self.spec);
        formatter.finish()
    }
}
