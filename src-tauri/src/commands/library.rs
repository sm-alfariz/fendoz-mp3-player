use std::path::Path;
use crate::db::queries::Track;

#[tauri::command]
pub fn scan_directory(path: String) -> Result<Vec<Track>, String> {
    let dir = Path::new(&path);
    if !dir.exists() || !dir.is_dir() {
        return Err("Invalid directory path".to_string());
    }

    let mut tracks = Vec::new();
    let extensions = ["mp3", "wav", "flac", "m4a"];

    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if extensions.contains(&ext_str.as_str()) {
                    match Track::from_file(&path) {
                        Ok(track) => tracks.push(track),
                        Err(_) => continue,
                    }
                }
            }
        }
    }

    Ok(tracks)
}

#[tauri::command]
pub fn get_track_metadata(path: String) -> Result<Track, String> {
    Track::from_file(Path::new(&path)).map_err(|e| e.to_string())
}
