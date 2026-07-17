use tauri::State;
use std::sync::Arc;
use crate::audio::player::{AudioPlayer, PlaybackState, PlaybackMode};
use crate::db::queries::Track;

pub struct PlayerState(pub Arc<AudioPlayer>);

#[tauri::command]
pub fn play(player: State<'_, PlayerState>, track_id: String, tracks: Vec<Track>) -> Result<(), String> {
    let track = tracks.iter().find(|t| t.id == track_id)
        .ok_or_else(|| format!("Track not found: {}", track_id))?;

    player.0.play(track.clone())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause(player: State<'_, PlayerState>) {
    player.0.pause();
}

#[tauri::command]
pub fn resume(player: State<'_, PlayerState>) {
    player.0.resume();
}

#[tauri::command]
pub fn stop(player: State<'_, PlayerState>) {
    player.0.stop();
}

#[tauri::command]
pub fn seek(player: State<'_, PlayerState>, position_ms: u64) {
    player.0.seek(position_ms);
}

#[tauri::command]
pub fn set_volume(player: State<'_, PlayerState>, volume: f32) {
    player.0.set_volume(volume);
}

#[tauri::command]
pub fn get_volume(player: State<'_, PlayerState>) -> f32 {
    player.0.get_volume()
}

#[tauri::command]
pub fn next_track(player: State<'_, PlayerState>, tracks: Vec<Track>) -> Result<Option<Track>, String> {
    let current = player.0.get_current_track();
    let mode = player.0.get_mode();

    let next = match mode {
        PlaybackMode::Shuffle => {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            if tracks.len() > 1 {
                let idx = rng.gen_range(0..tracks.len());
                Some(tracks[idx].clone())
            } else {
                tracks.first().cloned()
            }
        }
        PlaybackMode::LoopSingle => current.clone(),
        PlaybackMode::LoopAll => {
            if let Some(cur) = current {
                let idx = tracks.iter().position(|t| t.id == cur.id);
                if let Some(i) = idx {
                    let next_idx = (i + 1) % tracks.len();
                    Some(tracks[next_idx].clone())
                } else {
                    tracks.first().cloned()
                }
            } else {
                tracks.first().cloned()
            }
        }
    };

    if let Some(track) = next {
        player.0.play(track.clone()).map_err(|e| e.to_string())?;
        Ok(Some(track))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn prev_track(player: State<'_, PlayerState>, tracks: Vec<Track>) -> Result<Option<Track>, String> {
    let current = player.0.get_current_track();

    let prev = if let Some(cur) = current {
        let idx = tracks.iter().position(|t| t.id == cur.id);
        if let Some(i) = idx {
            let prev_idx = if i == 0 { tracks.len() - 1 } else { i - 1 };
            Some(tracks[prev_idx].clone())
        } else {
            tracks.first().cloned()
        }
    } else {
        tracks.first().cloned()
    };

    if let Some(track) = prev {
        player.0.play(track.clone()).map_err(|e| e.to_string())?;
        Ok(Some(track))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub fn get_playback_state(player: State<'_, PlayerState>) -> String {
    match player.0.get_state() {
        PlaybackState::Playing => "playing",
        PlaybackState::Paused => "paused",
        PlaybackState::Stopped => "stopped",
    }.to_string()
}

#[tauri::command]
pub fn set_playback_mode(player: State<'_, PlayerState>, mode: String) {
    let playback_mode = match mode.as_str() {
        "loop_single" => PlaybackMode::LoopSingle,
        "loop_all" => PlaybackMode::LoopAll,
        "shuffle" => PlaybackMode::Shuffle,
        _ => PlaybackMode::LoopAll,
    };
    player.0.set_mode(playback_mode);
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
    player.0.get_position()
}

#[tauri::command]
pub fn get_current_duration(player: State<'_, PlayerState>) -> u64 {
    player.0.get_duration()
}

#[tauri::command]
pub fn is_finished(player: State<'_, PlayerState>) -> bool {
    player.0.is_finished()
}
