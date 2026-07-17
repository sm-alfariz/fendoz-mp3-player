use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistTrack {
    pub playlist_id: i64,
    pub track_id: String,
    pub position: i64,
}

pub struct PlaylistStore {
    playlists: Mutex<Vec<Playlist>>,
    playlist_tracks: Mutex<Vec<PlaylistTrack>>,
    next_id: Mutex<i64>,
}

impl PlaylistStore {
    pub fn new() -> Self {
        Self {
            playlists: Mutex::new(Vec::new()),
            playlist_tracks: Mutex::new(Vec::new()),
            next_id: Mutex::new(1),
        }
    }
}

#[tauri::command]
pub fn get_playlists(store: State<'_, PlaylistStore>) -> Vec<Playlist> {
    store.playlists.lock().unwrap().clone()
}

#[tauri::command]
pub fn create_playlist(store: State<'_, PlaylistStore>, name: String) -> i64 {
    let mut playlists = store.playlists.lock().unwrap();
    let mut next_id = store.next_id.lock().unwrap();
    let id = *next_id;
    *next_id += 1;
    playlists.push(Playlist { id, name });
    id
}

#[tauri::command]
pub fn rename_playlist(store: State<'_, PlaylistStore>, id: i64, name: String) {
    let mut playlists = store.playlists.lock().unwrap();
    if let Some(playlist) = playlists.iter_mut().find(|p| p.id == id) {
        playlist.name = name;
    }
}

#[tauri::command]
pub fn delete_playlist(store: State<'_, PlaylistStore>, id: i64) {
    let mut playlists = store.playlists.lock().unwrap();
    playlists.retain(|p| p.id != id);

    let mut tracks = store.playlist_tracks.lock().unwrap();
    tracks.retain(|t| t.playlist_id != id);
}

#[tauri::command]
pub fn get_playlist_tracks(store: State<'_, PlaylistStore>, playlist_id: i64) -> Vec<PlaylistTrack> {
    let tracks = store.playlist_tracks.lock().unwrap();
    tracks.iter()
        .filter(|t| t.playlist_id == playlist_id)
        .cloned()
        .collect()
}

#[tauri::command]
pub fn add_to_playlist(store: State<'_, PlaylistStore>, playlist_id: i64, track_ids: Vec<String>) {
    let mut tracks = store.playlist_tracks.lock().unwrap();

    let max_pos = tracks.iter()
        .filter(|t| t.playlist_id == playlist_id)
        .map(|t| t.position)
        .max()
        .unwrap_or(-1);

    let mut position = max_pos + 1;
    for track_id in track_ids {
        // Check if track already exists in playlist
        let exists = tracks.iter().any(|t| t.playlist_id == playlist_id && t.track_id == track_id);
        if !exists {
            tracks.push(PlaylistTrack {
                playlist_id,
                track_id,
                position,
            });
            position += 1;
        }
    }
}

#[tauri::command]
pub fn remove_from_playlist(store: State<'_, PlaylistStore>, playlist_id: i64, track_ids: Vec<String>) {
    let mut tracks = store.playlist_tracks.lock().unwrap();
    tracks.retain(|t| !(t.playlist_id == playlist_id && track_ids.contains(&t.track_id)));
}

#[tauri::command]
pub fn reorder_playlist(store: State<'_, PlaylistStore>, playlist_id: i64, order: Vec<String>) {
    let mut tracks = store.playlist_tracks.lock().unwrap();

    // Remove existing tracks for this playlist
    tracks.retain(|t| t.playlist_id != playlist_id);

    // Add tracks in new order
    for (position, track_id) in order.iter().enumerate() {
        tracks.push(PlaylistTrack {
            playlist_id,
            track_id: track_id.clone(),
            position: position as i64,
        });
    }
}
