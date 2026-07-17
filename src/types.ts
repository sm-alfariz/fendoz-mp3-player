export interface Track {
  id: string;
  file_path: string;
  title: string;
  artist: string;
  album: string;
  duration_ms: number;
}

export interface Playlist {
  id: number;
  name: string;
  created_at: string;
}

export interface PlaylistTrack {
  playlist_id: number;
  track_id: string;
  position: number;
}

export interface Band {
  frequency: number;
  gain: number;
}

export interface EqPreset {
  name: string;
  gains: number[];
}

export interface LyricLine {
  timestamp_ms: number;
  text: string;
}

export type PlaybackMode = 'loop_single' | 'loop_all' | 'shuffle';
export type PlaybackState = 'playing' | 'paused' | 'stopped';
