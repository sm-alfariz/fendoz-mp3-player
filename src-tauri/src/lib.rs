mod commands;
mod audio;
mod db;

use std::sync::{Arc, Mutex};
use commands::playback::PlayerState;
use commands::equalizer::EqState;
use commands::playlist::PlaylistStore;
use audio::player::AudioPlayer;
use audio::dsp::Equalizer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize audio player
    let player = Arc::new(AudioPlayer::new());
    let eq = Arc::new(Mutex::new(Equalizer::new(44100.0)));
    let playlist_store = PlaylistStore::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PlayerState(player))
        .manage(EqState(eq))
        .manage(playlist_store)
        .invoke_handler(tauri::generate_handler![
            // Playback commands
            commands::playback::play,
            commands::playback::pause,
            commands::playback::resume,
            commands::playback::stop,
            commands::playback::seek,
            commands::playback::set_volume,
            commands::playback::get_volume,
            commands::playback::next_track,
            commands::playback::prev_track,
            commands::playback::get_playback_state,
            commands::playback::set_playback_mode,
            commands::playback::get_playback_mode,
            commands::playback::get_current_position,
            commands::playback::get_current_duration,
            commands::playback::is_finished,

            // Library commands
            commands::library::scan_directory,
            commands::library::get_track_metadata,

            // Equalizer commands
            commands::equalizer::get_equalizer_bands,
            commands::equalizer::set_equalizer_bands,
            commands::equalizer::get_eq_presets,
            commands::equalizer::load_eq_preset,
            commands::equalizer::reset_equalizer,

            // Lyrics commands
            commands::lyrics::get_lyrics,

            // Playlist commands
            commands::playlist::get_playlists,
            commands::playlist::create_playlist,
            commands::playlist::rename_playlist,
            commands::playlist::delete_playlist,
            commands::playlist::get_playlist_tracks,
            commands::playlist::add_to_playlist,
            commands::playlist::remove_from_playlist,
            commands::playlist::reorder_playlist,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
