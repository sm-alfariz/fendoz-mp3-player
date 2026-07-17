import { createContext, useContext, useState, useCallback, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LyricLine, Track } from '../types';

interface LyricsContextType {
  lyrics: LyricLine[];
  activeLineIndex: number;
  loadLyrics: (track: Track) => Promise<void>;
  updateActiveLine: (positionMs: number) => void;
}

const LyricsContext = createContext<LyricsContextType | undefined>(undefined);

export function LyricsProvider({ children }: { children: ReactNode }) {
  const [lyrics, setLyrics] = useState<LyricLine[]>([]);
  const [activeLineIndex, setActiveLineIndex] = useState(-1);

  const loadLyrics = useCallback(async (track: Track) => {
    const result = await invoke<LyricLine[] | null>('get_lyrics', { trackPath: track.file_path });
    setLyrics(result || []);
    setActiveLineIndex(-1);
  }, []);

  const updateActiveLine = useCallback((positionMs: number) => {
    if (lyrics.length === 0) {
      setActiveLineIndex(-1);
      return;
    }

    let index = -1;
    for (let i = 0; i < lyrics.length; i++) {
      if (lyrics[i].timestamp_ms <= positionMs) {
        index = i;
      } else {
        break;
      }
    }
    setActiveLineIndex(index);
  }, [lyrics]);

  return (
    <LyricsContext.Provider
      value={{
        lyrics,
        activeLineIndex,
        loadLyrics,
        updateActiveLine,
      }}
    >
      {children}
    </LyricsContext.Provider>
  );
}

export function useLyrics() {
  const context = useContext(LyricsContext);
  if (!context) {
    throw new Error('useLyrics must be used within a LyricsProvider');
  }
  return context;
}
