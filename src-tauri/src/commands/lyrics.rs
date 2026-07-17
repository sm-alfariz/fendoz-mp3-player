use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    pub timestamp_ms: u64,
    pub text: String,
}

#[tauri::command]
pub fn get_lyrics(track_path: String) -> Option<Vec<LyricLine>> {
    let path = Path::new(&track_path);
    let lrc_path = path.with_extension("lrc");

    if !lrc_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&lrc_path).ok()?;
    let mut lines = Vec::new();

    for line in content.lines() {
        if let Some(parsed) = parse_lrc_line(line) {
            lines.push(parsed);
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

fn parse_lrc_line(line: &str) -> Option<LyricLine> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Try to parse [mm:ss.xx] format
    if line.starts_with('[') {
        let closing = line.find(']')?;
        let time_str = &line[1..closing];

        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let minutes: u64 = parts[0].parse().ok()?;
        let seconds: f32 = parts[1].parse().ok()?;

        let timestamp_ms = minutes * 60_000 + (seconds * 1000.0) as u64;
        let text = line[closing + 1..].trim().to_string();

        Some(LyricLine { timestamp_ms, text })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lrc_line() {
        let line = "[01:23.45] Hello World";
        let parsed = parse_lrc_line(line).unwrap();
        assert_eq!(parsed.timestamp_ms, 83450);
        assert_eq!(parsed.text, "Hello World");
    }

    #[test]
    fn test_parse_lrc_line_empty() {
        assert!(parse_lrc_line("").is_none());
        assert!(parse_lrc_line("[invalid]").is_none());
    }
}
