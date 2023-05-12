use anyhow::{bail, Result};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::{fs::File, io::BufReader};

pub struct AudioManager {
    volume: f32,
    stream: Option<OutputStream>,
    handle: Option<OutputStreamHandle>,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            volume: 0.75,
            stream: None,
            handle: None,
        }
    }

    pub fn setup_stream(&mut self) -> Result<()> {
        let output: (OutputStream, OutputStreamHandle) = OutputStream::try_default()?;
        self.stream = Some(output.0);
        self.handle = Some(output.1);
        Ok(())
    }

    fn play_file(&self, path: &str) -> Result<()> {
        if let Some(handle) = &self.handle {
            return Ok(handle.play_raw(
                Decoder::new(BufReader::new(File::open(path)?))?
                    .amplify(self.volume)
                    .convert_samples(),
            )?);
        }

        bail!("No audio stream available")
    }

    pub fn mute(&mut self, muted: bool) {
        self.volume = if muted { 0.0 } else { 0.75 };
    }

    pub fn play_disconnected(&self) -> Result<()> {
        self.play_file("assets/disconnected.mp3")
    }
}
