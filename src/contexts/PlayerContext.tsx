import { createContext, useContext, useState, useEffect, useCallback, useRef, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Track, PlaybackMode } from '../types';

interface PlayerContextType {
  currentTrack: Track | null;
  position: number;
  duration: number;
  state: string;
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
  const [state, setState] = useState<string>('stopped');
  const [mode, setModeState] = useState<PlaybackMode>('loop_all');
  const [volume] = useState(0.8);
  const [tracks, setTracks] = useState<Track[]>([]);
  const tracksRef = useRef(tracks);
  tracksRef.current = tracks;

  // Poll position every 100ms
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const [pos, dur, st] = await Promise.all([
          invoke<number>('get_current_position'),
          invoke<number>('get_current_duration'),
          invoke<string>('get_playback_state'),
        ]);
        setPosition(pos);
        setDuration(dur);
        setState(st);
      } catch {}
    }, 100);
    return () => clearInterval(interval);
  }, []);

  const play = useCallback(async (track: Track) => {
    await invoke('play', { trackId: track.id, tracks: tracksRef.current });
    setCurrentTrack(track);
    setState('playing');
  }, []);

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
  }, []);

  const setVolume = useCallback(async (vol: number) => {
    await invoke('set_volume', { volume: vol });
  }, []);

  const nextTrack = useCallback(async () => {
    if (tracksRef.current.length === 0) return;
    const next = await invoke<Track | null>('next_track', { tracks: tracksRef.current });
    if (next) {
      setCurrentTrack(next);
      setState('playing');
    }
  }, []);

  const prevTrack = useCallback(async () => {
    if (tracksRef.current.length === 0) return;
    const prev = await invoke<Track | null>('prev_track', { tracks: tracksRef.current });
    if (prev) {
      setCurrentTrack(prev);
      setState('playing');
    }
  }, []);

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
