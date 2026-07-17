use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::io::BufReader;
use std::fs::File;
use crate::db::queries::Track;
use std::thread;
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
}

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
    SetMode(PlaybackMode),
    Quit,
}

pub struct AudioPlayer {
    command_tx: mpsc::Sender<PlayerCommand>,
    state: Arc<Mutex<PlaybackState>>,
    mode: Arc<Mutex<PlaybackMode>>,
    volume: Arc<Mutex<f32>>,
    current_track: Arc<Mutex<Option<Track>>>,
    position_ms: Arc<Mutex<u64>>,
    duration_ms: Arc<Mutex<u64>>,
    is_finished: Arc<AtomicBool>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel();

        let state = Arc::new(Mutex::new(PlaybackState::Stopped));
        let mode = Arc::new(Mutex::new(PlaybackMode::LoopAll));
        let volume = Arc::new(Mutex::new(0.8f32));
        let current_track = Arc::new(Mutex::new(None::<Track>));
        let position_ms = Arc::new(Mutex::new(0u64));
        let duration_ms = Arc::new(Mutex::new(0u64));
        let is_finished = Arc::new(AtomicBool::new(true));

        let state_clone = state.clone();
        let mode_clone = mode.clone();
        let volume_clone = volume.clone();
        let current_track_clone = current_track.clone();
        let position_ms_clone = position_ms.clone();
        let duration_ms_clone = duration_ms.clone();
        let is_finished_clone = is_finished.clone();

        // Spawn audio thread
        thread::spawn(move || {
            audio_thread(
                command_rx,
                state_clone,
                mode_clone,
                volume_clone,
                current_track_clone,
                position_ms_clone,
                duration_ms_clone,
                is_finished_clone,
            );
        });

        Self {
            command_tx,
            state,
            mode,
            volume,
            current_track,
            position_ms,
            duration_ms,
            is_finished,
        }
    }

    pub fn play(&self, track: Track) -> Result<(), String> {
        self.command_tx.send(PlayerCommand::Play(track)).map_err(|e| e.to_string())
    }

    pub fn pause(&self) {
        let _ = self.command_tx.send(PlayerCommand::Pause);
    }

    pub fn resume(&self) {
        let _ = self.command_tx.send(PlayerCommand::Resume);
    }

    pub fn stop(&self) {
        let _ = self.command_tx.send(PlayerCommand::Stop);
    }

    pub fn seek(&self, position_ms: u64) {
        let _ = self.command_tx.send(PlayerCommand::Seek(position_ms));
    }

    pub fn set_volume(&self, volume: f32) {
        let _ = self.command_tx.send(PlayerCommand::SetVolume(volume));
    }

    pub fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }

    pub fn get_state(&self) -> PlaybackState {
        self.state.lock().unwrap().clone()
    }

    pub fn get_mode(&self) -> PlaybackMode {
        self.mode.lock().unwrap().clone()
    }

    pub fn set_mode(&self, mode: PlaybackMode) {
        *self.mode.lock().unwrap() = mode;
    }

    pub fn get_current_track(&self) -> Option<Track> {
        self.current_track.lock().unwrap().clone()
    }

    pub fn get_position(&self) -> u64 {
        *self.position_ms.lock().unwrap()
    }

    pub fn get_duration(&self) -> u64 {
        *self.duration_ms.lock().unwrap()
    }

    pub fn is_finished(&self) -> bool {
        self.is_finished.load(Ordering::Relaxed)
    }
}

fn audio_thread(
    command_rx: mpsc::Receiver<PlayerCommand>,
    state: Arc<Mutex<PlaybackState>>,
    mode: Arc<Mutex<PlaybackMode>>,
    volume: Arc<Mutex<f32>>,
    current_track: Arc<Mutex<Option<Track>>>,
    position_ms: Arc<Mutex<u64>>,
    duration_ms: Arc<Mutex<u64>>,
    is_finished: Arc<AtomicBool>,
) {
    use rodio::{Decoder, OutputStream, Sink};

    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut current_sink: Option<Sink> = None;

    loop {
        // Check for commands
        while let Ok(cmd) = command_rx.try_recv() {
            match cmd {
                PlayerCommand::Play(track) => {
                    // Stop current playback
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }

                    let path = std::path::Path::new(&track.file_path);
                    match File::open(path) {
                        Ok(file) => {
                            match Decoder::new(BufReader::new(file)) {
                                Ok(source) => {
                                    // Note: rodio doesn't expose duration directly
                                    // We'll get it from the metadata
                                    let dur = 0u64; // TODO: get from metadata
                                    *duration_ms.lock().unwrap() = dur;
                                    *position_ms.lock().unwrap() = 0;

                                    let sink = Sink::try_new(&stream_handle).unwrap();
                                    let vol = *volume.lock().unwrap();
                                    sink.set_volume(vol);
                                    sink.append(source);

                                    current_sink = Some(sink);
                                    *current_track.lock().unwrap() = Some(track);
                                    *state.lock().unwrap() = PlaybackState::Playing;
                                    is_finished.store(false, Ordering::Relaxed);
                                }
                                Err(e) => eprintln!("Failed to decode: {}", e),
                            }
                        }
                        Err(e) => eprintln!("Failed to open file: {}", e),
                    }
                }
                PlayerCommand::Pause => {
                    if let Some(sink) = &current_sink {
                        sink.pause();
                        *state.lock().unwrap() = PlaybackState::Paused;
                    }
                }
                PlayerCommand::Resume => {
                    if let Some(sink) = &current_sink {
                        sink.play();
                        *state.lock().unwrap() = PlaybackState::Playing;
                    }
                }
                PlayerCommand::Stop => {
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    *state.lock().unwrap() = PlaybackState::Stopped;
                    is_finished.store(true, Ordering::Relaxed);
                }
                PlayerCommand::Seek(pos) => {
                    if let Some(sink) = &current_sink {
                        let _ = sink.try_seek(std::time::Duration::from_millis(pos));
                        *position_ms.lock().unwrap() = pos;
                    }
                }
                PlayerCommand::SetVolume(vol) => {
                    let v = vol.clamp(0.0, 1.0);
                    *volume.lock().unwrap() = v;
                    if let Some(sink) = &current_sink {
                        sink.set_volume(v);
                    }
                }
                PlayerCommand::SetMode(m) => {
                    *mode.lock().unwrap() = m;
                }
                PlayerCommand::Quit => {
                    if let Some(sink) = current_sink.take() {
                        sink.stop();
                    }
                    return;
                }
            }
        }

        // Update position if playing
        if *state.lock().unwrap() == PlaybackState::Playing {
            if let Some(sink) = &current_sink {
                if sink.empty() {
                    is_finished.store(true, Ordering::Relaxed);
                    *state.lock().unwrap() = PlaybackState::Stopped;
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for AudioPlayer {}
unsafe impl Sync for AudioPlayer {}
