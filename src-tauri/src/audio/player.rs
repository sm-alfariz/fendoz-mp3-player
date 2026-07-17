use std::sync::{Arc, atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering}};
use std::fs::File;
use std::sync::mpsc;
use crate::db::queries::Track;
use std::thread;

const STATE_STOPPED: u8 = 0;
const STATE_PLAYING: u8 = 1;
const STATE_PAUSED: u8 = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackMode {
    LoopSingle,
    LoopAll,
    Shuffle,
}

enum PlayerCommand {
    Play(Track),
    Pause,
    Resume,
    Stop,
    Seek(u64),
    SetVolume(f32),
    Quit,
}

pub struct AudioPlayer {
    command_tx: mpsc::Sender<PlayerCommand>,
    state: Arc<AtomicU8>,
    mode: Arc<AtomicU8>,
    is_finished: Arc<AtomicBool>,
    position_ms: Arc<AtomicU64>,
    duration_ms: Arc<AtomicU64>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let state = Arc::new(AtomicU8::new(STATE_STOPPED));
        let mode = Arc::new(AtomicU8::new(1));
        let is_finished = Arc::new(AtomicBool::new(true));
        let position_ms = Arc::new(AtomicU64::new(0));
        let duration_ms = Arc::new(AtomicU64::new(0));

        let (s2, m2, f2, p2, d2) = (state.clone(), mode.clone(), is_finished.clone(), position_ms.clone(), duration_ms.clone());
        thread::spawn(move || audio_thread(command_rx, s2, m2, f2, p2, d2));

        Self { command_tx, state, mode, is_finished, position_ms, duration_ms }
    }

    pub fn play(&self, track: Track) -> Result<(), String> { self.command_tx.send(PlayerCommand::Play(track)).map_err(|e| e.to_string()) }
    pub fn pause(&self) { let _ = self.command_tx.send(PlayerCommand::Pause); }
    pub fn resume(&self) { let _ = self.command_tx.send(PlayerCommand::Resume); }
    pub fn stop(&self) { let _ = self.command_tx.send(PlayerCommand::Stop); }
    pub fn seek(&self, position_ms: u64) { let _ = self.command_tx.send(PlayerCommand::Seek(position_ms)); }
    pub fn set_volume(&self, volume: f32) { let _ = self.command_tx.send(PlayerCommand::SetVolume(volume)); }
    pub fn get_state(&self) -> u8 { self.state.load(Ordering::Relaxed) }
    pub fn get_mode(&self) -> PlaybackMode { match self.mode.load(Ordering::Relaxed) { 0 => PlaybackMode::LoopSingle, 2 => PlaybackMode::Shuffle, _ => PlaybackMode::LoopAll } }
    pub fn set_mode(&self, mode: PlaybackMode) { self.mode.store(match mode { PlaybackMode::LoopSingle => 0, PlaybackMode::LoopAll => 1, PlaybackMode::Shuffle => 2 }, Ordering::Relaxed); }
    pub fn get_current_position(&self) -> u64 { self.position_ms.load(Ordering::Relaxed) }
    pub fn get_duration(&self) -> u64 { self.duration_ms.load(Ordering::Relaxed) }
    pub fn is_finished(&self) -> bool { self.is_finished.load(Ordering::Relaxed) }
}

/// Decode file to f32 samples using ffmpeg
fn decode_with_ffmpeg(path: &std::path::Path) -> Result<(Vec<f32>, u32, u16), String> {
    use ffmpeg_next::format;
    use ffmpeg_next::codec;
    use ffmpeg_next::software::resampling;
    use ffmpeg_next::util::format::sample::{self, Sample};

    ffmpeg_next::init().map_err(|e| format!("ffmpeg init: {}", e))?;

    let mut ictx = format::input(path).map_err(|e| format!("ffmpeg open: {}", e))?;

    let input = ictx.streams().best(ffmpeg_next::media::Type::Audio)
        .ok_or("No audio stream")?;
    let stream_index = input.index();

    let params = input.parameters();
    let mut decoder = codec::context::Context::from_parameters(params)
        .map_err(|e| format!("context: {}", e))?
        .decoder()
        .audio()
        .map_err(|e| format!("decoder: {}", e))?;

    let in_sample_rate = decoder.rate();
    let in_channels = decoder.channels();
    let channel_layout = decoder.channel_layout();

    eprintln!("[ffmpeg] Input: {} Hz, {} ch", in_sample_rate, in_channels);

    let mut all_samples: Vec<f32> = Vec::new();
    let mut frame = ffmpeg_next::frame::Audio::empty();
    let mut output_frame = ffmpeg_next::frame::Audio::empty();

    let target_format = Sample::F32(sample::Type::Planar);
    let target_layout = channel_layout;

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index { continue; }

        decoder.send_packet(&packet).map_err(|e| e.to_string())?;
        while decoder.receive_frame(&mut frame).is_ok() {
            let mut resampler = resampling::Context::get(
                decoder.format(), channel_layout, in_sample_rate,
                target_format, target_layout, in_sample_rate,
            ).map_err(|e| format!("resampler: {}", e))?;

            resampler.run(&frame, &mut output_frame).map_err(|e| e.to_string())?;

            let ch = output_frame.channels() as usize;
            let samples_per_ch = output_frame.samples();
            for s in 0..samples_per_ch {
                for c in 0..ch {
                    let data = output_frame.data(c);
                    let offset = s * 4;
                    if offset + 4 <= data.len() {
                        let sample = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
                        all_samples.push(sample);
                    }
                }
            }
        }
    }

    // Flush
    decoder.send_eof().ok();
    while decoder.receive_frame(&mut frame).is_ok() {
        let mut resampler = resampling::Context::get(
            decoder.format(), channel_layout, in_sample_rate,
            target_format, target_layout, in_sample_rate,
        ).map_err(|e| e.to_string())?;
        resampler.run(&frame, &mut output_frame).map_err(|e| e.to_string())?;

        let ch = output_frame.channels() as usize;
        let samples_per_ch = output_frame.samples();
        for s in 0..samples_per_ch {
            for c in 0..ch {
                let data = output_frame.data(c);
                let offset = s * 4;
                if offset + 4 <= data.len() {
                    let sample = f32::from_le_bytes([data[offset], data[offset+1], data[offset+2], data[offset+3]]);
                    all_samples.push(sample);
                }
            }
        }
    }

    eprintln!("[ffmpeg] Decoded {} total samples", all_samples.len());
    Ok((all_samples, in_sample_rate, in_channels))
}

struct AudioBuffer {
    samples: Vec<f32>,
    read_pos: usize,
    sample_rate: u32,
    channels: u16,
}

impl AudioBuffer {
    fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        Self { samples, read_pos: 0, sample_rate, channels }
    }

    fn fill(&mut self, output: &mut [f32], volume: f32) {
        for sample in output.iter_mut() {
            if self.read_pos < self.samples.len() {
                *sample = self.samples[self.read_pos] * volume;
                self.read_pos += 1;
            } else {
                *sample = 0.0;
            }
        }
    }

    fn seek_to_sample(&mut self, sample_pos: usize) {
        self.read_pos = sample_pos.min(self.samples.len());
    }

    fn position_ms(&self) -> u64 {
        if self.channels == 0 || self.sample_rate == 0 { return 0; }
        (self.read_pos as u64 * 1000) / (self.sample_rate as u64 * self.channels as u64)
    }

    fn duration_ms(&self) -> u64 {
        if self.channels == 0 || self.sample_rate == 0 { return 0; }
        (self.samples.len() as u64 * 1000) / (self.sample_rate as u64 * self.channels as u64)
    }

    fn is_empty(&self) -> bool { self.read_pos >= self.samples.len() }
}

fn audio_thread(
    command_rx: mpsc::Receiver<PlayerCommand>,
    state: Arc<AtomicU8>,
    _mode: Arc<AtomicU8>,
    is_finished: Arc<AtomicBool>,
    position_ms: Arc<AtomicU64>,
    duration_ms: Arc<AtomicU64>,
) {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{SampleFormat, StreamConfig, BufferSize};

    let host = cpal::default_host();
    let device = host.default_output_device().expect("No output device");
    let supported = device.default_output_config().expect("No default output config");

    let sample_format = supported.sample_format();
    let device_sample_rate = supported.sample_rate().0;

    // LARGE buffer to prevent Bluetooth/PipeWire underruns
    let config = StreamConfig {
        channels: supported.channels(),
        sample_rate: supported.sample_rate(),
        buffer_size: BufferSize::Fixed(4096),
    };

    eprintln!("[audio] Device: {} Hz, {} ch", device_sample_rate, supported.channels());

    let audio_buf: Arc<std::sync::Mutex<Option<AudioBuffer>>> = Arc::new(std::sync::Mutex::new(None));
    let paused = Arc::new(AtomicBool::new(false));
    let volume = Arc::new(std::sync::Mutex::new(0.8f32));

    let buf_ref = audio_buf.clone();
    let vol_ref = volume.clone();
    let pause_ref = paused.clone();

    let stream = match sample_format {
        SampleFormat::F32 => {
            device.build_output_stream(&config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let vol = vol_ref.lock().map(|v| *v).unwrap_or(0.8);
                    let is_paused = pause_ref.load(Ordering::Relaxed);
                    if let Ok(mut guard) = buf_ref.try_lock() {
                        if let Some(ref mut buf) = *guard {
                            if is_paused { for s in data.iter_mut() { *s = 0.0; } }
                            else { buf.fill(data, vol); return; }
                        }
                    }
                    for s in data.iter_mut() { *s = 0.0; }
                },
                move |err| eprintln!("[audio] Error: {}", err), None)
        }
        SampleFormat::U16 => {
            let buf_ref2 = buf_ref.clone();
            let vol_ref2 = volume.clone();
            let pause_ref2 = paused.clone();
            device.build_output_stream(&config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    let vol = vol_ref2.lock().map(|v| *v).unwrap_or(0.8);
                    let is_paused = pause_ref2.load(Ordering::Relaxed);
                    if let Ok(mut guard) = buf_ref2.try_lock() {
                        if let Some(ref mut buf) = *guard {
                            for s in data.iter_mut() {
                                if is_paused { *s = i16::MIN as u16; }
                                else {
                                    let mut f = [0.0f32];
                                    buf.fill(&mut f, vol);
                                    *s = ((f[0] * 32767.0) as i16 as u16).wrapping_add(i16::MIN as u16);
                                }
                            }
                            return;
                        }
                    }
                    for s in data.iter_mut() { *s = i16::MIN as u16; }
                },
                move |err| eprintln!("[audio] Error: {}", err), None)
        }
        _ => panic!("Unsupported format"),
    }.expect("Failed to build stream");

    stream.play().expect("Failed to play");

    loop {
        let cmd = command_rx.recv_timeout(std::time::Duration::from_millis(10));
        if let Ok(cmd) = cmd {
            match cmd {
                PlayerCommand::Play(track) => {
                    let path = std::path::Path::new(&track.file_path);
                    eprintln!("[audio] Playing: {}", path.display());
                    match decode_with_ffmpeg(path) {
                        Ok((samples, _rate, channels)) => {
                            let buf = AudioBuffer::new(samples, device_sample_rate, channels);
                            eprintln!("[audio] {} ms of audio ready", buf.duration_ms());
                            duration_ms.store(buf.duration_ms(), Ordering::Relaxed);
                            position_ms.store(0, Ordering::Relaxed);
                            if let Ok(mut guard) = audio_buf.lock() { *guard = Some(buf); }
                            state.store(STATE_PLAYING, Ordering::Relaxed);
                            paused.store(false, Ordering::Relaxed);
                            is_finished.store(false, Ordering::Relaxed);
                        }
                        Err(e) => eprintln!("[audio] Decode FAILED: {}", e),
                    }
                }
                PlayerCommand::Pause => { paused.store(true, Ordering::Relaxed); state.store(STATE_PAUSED, Ordering::Relaxed); }
                PlayerCommand::Resume => { paused.store(false, Ordering::Relaxed); state.store(STATE_PLAYING, Ordering::Relaxed); }
                PlayerCommand::Stop => {
                    if let Ok(mut guard) = audio_buf.lock() { *guard = None; }
                    state.store(STATE_STOPPED, Ordering::Relaxed);
                    position_ms.store(0, Ordering::Relaxed);
                    is_finished.store(true, Ordering::Relaxed);
                }
                PlayerCommand::Seek(pos_ms) => {
                    if let Ok(mut guard) = audio_buf.lock() {
                        if let Some(ref mut buf) = *guard {
                            let sp = (pos_ms * buf.sample_rate as u64 * buf.channels as u64 / 1000) as usize;
                            buf.seek_to_sample(sp);
                            position_ms.store(pos_ms, Ordering::Relaxed);
                        }
                    }
                }
                PlayerCommand::SetVolume(vol) => { if let Ok(mut v) = volume.lock() { *v = vol.clamp(0.0, 1.0); } }
                PlayerCommand::Quit => { drop(stream); return; }
            }
        }

        if state.load(Ordering::Relaxed) == STATE_PLAYING {
            if let Ok(guard) = audio_buf.try_lock() {
                if let Some(ref buf) = *guard {
                    position_ms.store(buf.position_ms(), Ordering::Relaxed);
                    if buf.is_empty() { is_finished.store(true, Ordering::Relaxed); state.store(STATE_STOPPED, Ordering::Relaxed); }
                }
            }
        }
    }
}

impl Default for AudioPlayer { fn default() -> Self { Self::new() } }
unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}
