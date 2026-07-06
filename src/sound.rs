use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub enum SoundEvent {
    Eat,
    Hatch,
    Fade,
    Chirp,
}

pub struct SoundPlayer {
    enabled: bool,
    chirp: PathBuf,
    eat: PathBuf,
    hatch: PathBuf,
    fade: PathBuf,
    last_event: Instant,
    last_ambient: Instant,
}

impl SoundPlayer {
    pub fn new(enabled: bool) -> Self {
        let dir = std::env::temp_dir().join("thronglets-sounds");
        if enabled {
            let _ = std::fs::create_dir_all(&dir);
            let _ = write_chirp(&dir.join("chirp.wav"), 1320.0, 1900.0, 90, 0.28);
            let _ = write_chirp(&dir.join("eat.wav"), 940.0, 1420.0, 110, 0.24);
            let _ = write_chirp(&dir.join("hatch.wav"), 880.0, 1760.0, 180, 0.30);
            let _ = write_chirp(&dir.join("fade.wav"), 620.0, 380.0, 260, 0.22);
        }
        let now = Instant::now();
        SoundPlayer {
            enabled,
            chirp: dir.join("chirp.wav"),
            eat: dir.join("eat.wav"),
            hatch: dir.join("hatch.wav"),
            fade: dir.join("fade.wav"),
            last_event: now,
            last_ambient: now,
        }
    }

    pub fn play(&mut self, event: SoundEvent) {
        if !self.enabled || self.last_event.elapsed() < Duration::from_millis(90) {
            return;
        }
        self.last_event = Instant::now();
        let path = match event {
            SoundEvent::Eat => &self.eat,
            SoundEvent::Hatch => &self.hatch,
            SoundEvent::Fade => &self.fade,
            SoundEvent::Chirp => &self.chirp,
        };
        play_file(path);
    }

    pub fn ambient_population(&mut self, population: usize, tick: u64) {
        if !self.enabled || population == 0 {
            return;
        }
        let interval_ms = (2200u64.saturating_sub((population as u64).min(35) * 38)).max(850);
        if self.last_ambient.elapsed() >= Duration::from_millis(interval_ms)
            && tick % 17 < (population as u64).min(6)
        {
            self.last_ambient = Instant::now();
            self.play(SoundEvent::Chirp);
        }
    }
}

fn play_file(path: &PathBuf) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("afplay").arg(path).spawn();
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        print!("\x07");
    }
}

fn write_chirp(
    path: &PathBuf,
    start_hz: f32,
    end_hz: f32,
    duration_ms: u32,
    volume: f32,
) -> std::io::Result<()> {
    let sample_rate = 44_100u32;
    let samples = sample_rate as usize * duration_ms as usize / 1000;
    let mut data = Vec::with_capacity(samples * 2);
    let mut phase = 0.0f32;
    for i in 0..samples {
        let t = i as f32 / samples.max(1) as f32;
        let freq = start_hz + (end_hz - start_hz) * t;
        phase += std::f32::consts::TAU * freq / sample_rate as f32;
        let envelope = if t < 0.18 {
            t / 0.18
        } else {
            ((1.0 - t) / 0.82).max(0.0)
        };
        let wobble = (std::f32::consts::TAU * 18.0 * t).sin() * 0.18 + 0.82;
        let sample = (phase.sin() * envelope * wobble * volume * i16::MAX as f32) as i16;
        data.extend_from_slice(&sample.to_le_bytes());
    }
    write_wav(path, sample_rate, &data)
}

fn write_wav(path: &PathBuf, sample_rate: u32, data: &[u8]) -> std::io::Result<()> {
    let data_len = data.len() as u32;
    let mut out = Vec::with_capacity(44 + data.len());
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    out.extend_from_slice(data);
    std::fs::write(path, out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_chirp_is_a_wav_file() {
        let path = std::env::temp_dir().join("thronglets-test-chirp.wav");
        write_chirp(&path, 900.0, 1400.0, 40, 0.2).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.starts_with(b"RIFF"));
        assert_eq!(&bytes[8..12], b"WAVE");
        let _ = std::fs::remove_file(path);
    }
}
