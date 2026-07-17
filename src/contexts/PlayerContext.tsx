import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { Track, PlaybackState, PlaybackMode } from '../types';

interface PlayerContextType {
  currentTrack: Track | null;
  position: number;
  duration: number;
  state: PlaybackState;
  mode: PlaybackMode;
  volume: number;
  tracks: Track[];
  play: (track: Track) => void;
  pause: () => void;
  resume: () => void;
  stop: () => void;
  seek: (positionMs: number) => void;
  setVolume: (volume: number) => void;
  nextTrack: () => void;
  prevTrack: () => void;
  changeMode: (mode: PlaybackMode) => void;
  setTracks: (tracks: Track[]) => void;
}

const PlayerContext = createContext<PlayerContextType | undefined>(undefined);

export function PlayerProvider({ children }: { children: ReactNode }) {
  const [currentTrack, setCurrentTrack] = useState<Track | null>(null);
  const [position, setPosition] = useState(0);
  const [duration, setDuration] = useState(0);
  const [state, setState] = useState<PlaybackState>('stopped');
  const [mode, setModeState] = useState<PlaybackMode>('loop_all');
  const [volume, setVolumeState] = useState(0.8);
  const [tracks, setTracks] = useState<Track[]>([]);

  // Listen for position updates from Rust
  useEffect(() => {
    const unlisten = listen<{ position_ms: number; duration_ms: number; is_playing: boolean }>(
      'playback-position',
      (event) => {
        setPosition(event.payload.position_ms);
        setDuration(event.payload.duration_ms);
      }
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for track changes
  useEffect(() => {
    const unlisten = listen<{ track: Track }>('track-changed', (event) => {
      setCurrentTrack(event.payload.track);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Listen for playback ended
  useEffect(() => {
    const unlisten = listen<{ next_track_id?: string }>('playback-ended', () => {
      nextTrack();
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [tracks, mode, currentTrack]);

  const play = useCallback(async (track: Track) => {
    await invoke('play', { trackId: track.id, tracks });
    setCurrentTrack(track);
    setState('playing');
  }, [tracks]);

  const pause = useCallback(async () => {
    await invoke('pause');
    setState('paused');
  }, []);

  const resume = useCallback(async () => {
    await invoke('resume');
    setState('playing');
  }, []);

  const stop = useCallback(async () => {
    await invoke('stop');
    setState('stopped');
    setPosition(0);
  }, []);

  const seek = useCallback(async (positionMs: number) => {
    await invoke('seek', { positionMs });
    setPosition(positionMs);
  }, []);

  const setVolume = useCallback(async (vol: number) => {
    await invoke('set_volume', { volume: vol });
    setVolumeState(vol);
  }, []);

  const nextTrack = useCallback(async () => {
    if (tracks.length === 0) return;
    const next = await invoke<Track | null>('next_track', { tracks });
    if (next) {
      setCurrentTrack(next);
      setState('playing');
    }
  }, [tracks]);

  const prevTrack = useCallback(async () => {
    if (tracks.length === 0) return;
    const prev = await invoke<Track | null>('prev_track', { tracks });
    if (prev) {
      setCurrentTrack(prev);
      setState('playing');
    }
  }, [tracks]);

  const changeMode = useCallback(async (newMode: PlaybackMode) => {
    await invoke('set_playback_mode', { mode: newMode });
    setModeState(newMode);
  }, []);

  return (
    <PlayerContext.Provider
      value={{
        currentTrack,
        position,
        duration,
        state,
        mode,
        volume,
        tracks,
        play,
        pause,
        resume,
        stop,
        seek,
        setVolume,
        nextTrack,
        prevTrack,
        changeMode,
        setTracks,
      }}
    >
      {children}
    </PlayerContext.Provider>
  );
}

export function usePlayer() {
  const context = useContext(PlayerContext);
  if (!context) {
    throw new Error('usePlayer must be used within a PlayerProvider');
  }
  return context;
}
