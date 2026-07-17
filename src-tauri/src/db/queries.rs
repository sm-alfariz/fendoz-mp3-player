use std::path::Path;
use lofty::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub file_path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
}

impl Track {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = lofty::read_from_path(path)?;

        let properties = file.properties();
        let duration_ms = properties.duration().as_millis() as u64;

        let title = file.primary_tag()
            .and_then(|tag| tag.title().map(|s| s.to_string()))
            .unwrap_or_else(|| path.file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string());

        let artist = file.primary_tag()
            .and_then(|tag| tag.artist().map(|s| s.to_string()))
            .unwrap_or_else(|| "Unknown Artist".to_string());

        let album = file.primary_tag()
            .and_then(|tag| tag.album().map(|s| s.to_string()))
            .unwrap_or_else(|| "Unknown Album".to_string());

        let id = path.to_string_lossy().to_string();

        Ok(Track {
            id,
            file_path: path.to_string_lossy().to_string(),
            title,
            artist,
            album,
            duration_ms,
        })
    }
}
