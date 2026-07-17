use tauri::State;
use std::sync::Arc;
use crate::audio::player::{AudioPlayer, PlaybackMode};
use crate::db::queries::Track;

const STATE_PLAYING: u8 = 1;

pub struct PlayerState(pub Arc<AudioPlayer>);

#[tauri::command]
pub fn play(player: State<'_, PlayerState>, track_id: String, tracks: Vec<Track>) -> Result<(), String> {
    let track = tracks.iter().find(|t| t.id == track_id)
        .ok_or_else(|| format!("Track not found: {}", track_id))?;
    player.0.play(track.clone())
}

#[tauri::command]
pub fn pause(player: State<'_, PlayerState>) { player.0.pause(); }

#[tauri::command]
pub fn resume(player: State<'_, PlayerState>) { player.0.resume(); }

#[tauri::command]
pub fn stop(player: State<'_, PlayerState>) { player.0.stop(); }

#[tauri::command]
pub fn seek(player: State<'_, PlayerState>, position_ms: u64) { player.0.seek(position_ms); }

#[tauri::command]
pub fn set_volume(player: State<'_, PlayerState>, volume: f32) { player.0.set_volume(volume); }

#[tauri::command]
pub fn get_volume(_player: State<'_, PlayerState>) -> f32 { 0.8 }

#[tauri::command]
pub fn next_track(player: State<'_, PlayerState>, tracks: Vec<Track>) -> Result<Option<Track>, String> {
    if tracks.is_empty() { return Ok(None); }
    let mode = player.0.get_mode();

    let next = match mode {
        PlaybackMode::Shuffle => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            Some(tracks[rng.gen_range(0..tracks.len())].clone())
        }
        _ => tracks.first().cloned(),
    };

    if let Some(track) = next {
        player.0.play(track.clone())?;
        Ok(Some(track))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn prev_track(player: State<'_, PlayerState>, tracks: Vec<Track>) -> Result<Option<Track>, String> {
    if tracks.is_empty() { return Ok(None); }
    if let Some(track) = tracks.first().cloned() {
        player.0.play(track.clone())?;
        Ok(Some(track))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn get_playback_state(player: State<'_, PlayerState>) -> String {
    match player.0.get_state() {
        STATE_PLAYING => "playing",
        2 => "paused",
        _ => "stopped",
    }.to_string()
}

#[tauri::command]
pub fn set_playback_mode(player: State<'_, PlayerState>, mode: String) {
    let m = match mode.as_str() {
        "loop_single" => PlaybackMode::LoopSingle,
        "shuffle" => PlaybackMode::Shuffle,
        _ => PlaybackMode::LoopAll,
    };
    player.0.set_mode(m);
}

#[tauri::command]
pub fn get_playback_mode(player: State<'_, PlayerState>) -> String {
    match player.0.get_mode() {
        PlaybackMode::LoopSingle => "loop_single",
        PlaybackMode::LoopAll => "loop_all",
        PlaybackMode::Shuffle => "shuffle",
    }.to_string()
}

#[tauri::command]
pub fn get_current_position(player: State<'_, PlayerState>) -> u64 {
    player.0.get_current_position()
}

#[tauri::command]
pub fn get_current_duration(player: State<'_, PlayerState>) -> u64 {
    player.0.get_duration()
}

#[tauri::command]
pub fn is_finished(player: State<'_, PlayerState>) -> bool {
    player.0.is_finished()
}
