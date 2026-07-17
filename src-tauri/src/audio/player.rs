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

fn audio_thread(
    command_rx: mpsc::Receiver<PlayerCommand>,
    state: Arc<AtomicU8>,
    _mode: Arc<AtomicU8>,
    is_finished: Arc<AtomicBool>,
    position_ms: Arc<AtomicU64>,
    duration_ms: Arc<AtomicU64>,
) {
    use rodio::{Decoder, OutputStream, Sink, Source};

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut current_sink: Option<Sink> = None;

    loop {
        let cmd = command_rx.recv_timeout(std::time::Duration::from_millis(10));
        if let Ok(cmd) = cmd {
            match cmd {
                PlayerCommand::Play(track) => {
                    if let Some(sink) = current_sink.take() { sink.stop(); }

                    let path = std::path::Path::new(&track.file_path);
                    eprintln!("[audio] Playing: {}", path.display());

                    match File::open(path) {
                        Ok(file) => {
                            match Decoder::new(std::io::BufReader::new(file)) {
                                Ok(source) => {
                                    let dur = source.total_duration().map(|d| d.as_millis() as u64).unwrap_or(0);
                                    duration_ms.store(dur, Ordering::Relaxed);
                                    position_ms.store(0, Ordering::Relaxed);

                                    let sink = Sink::try_new(&stream_handle).unwrap();
                                    sink.set_volume(0.8);
                                    sink.append(source);

                                    current_sink = Some(sink);
                                    state.store(STATE_PLAYING, Ordering::Relaxed);
                                    is_finished.store(false, Ordering::Relaxed);
                                    eprintln!("[audio] Playback started, {}ms", dur);
                                }
                                Err(e) => eprintln!("[audio] Decoder error: {}", e),
                            }
                        }
                        Err(e) => eprintln!("[audio] File open error: {}", e),
                    }
                }
                PlayerCommand::Pause => {
                    if let Some(ref sink) = current_sink { sink.pause(); }
                    state.store(STATE_PAUSED, Ordering::Relaxed);
                }
                PlayerCommand::Resume => {
                    if let Some(ref sink) = current_sink { sink.play(); }
                    state.store(STATE_PLAYING, Ordering::Relaxed);
                }
                PlayerCommand::Stop => {
                    if let Some(sink) = current_sink.take() { sink.stop(); }
                    state.store(STATE_STOPPED, Ordering::Relaxed);
                    position_ms.store(0, Ordering::Relaxed);
                    is_finished.store(true, Ordering::Relaxed);
                }
                PlayerCommand::Seek(pos_ms) => {
                    if let Some(ref sink) = current_sink { let _ = sink.try_seek(std::time::Duration::from_millis(pos_ms)); }
                }
                PlayerCommand::SetVolume(vol) => {
                    if let Some(ref sink) = current_sink { sink.set_volume(vol.clamp(0.0, 1.0)); }
                }
                PlayerCommand::Quit => {
                    if let Some(sink) = current_sink.take() { sink.stop(); }
                    return;
                }
            }
        }

        // Update position (don't check empty — rodio reports empty before audio starts)
        if state.load(Ordering::Relaxed) == STATE_PLAYING {
            if let Some(ref sink) = current_sink {
                let pos = sink.get_pos().as_millis() as u64;
                position_ms.store(pos, Ordering::Relaxed);
            }
        }
    }
}

impl Default for AudioPlayer { fn default() -> Self { Self::new() } }
unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}
