import React, { useEffect, useRef } from 'react';
import { useLyrics } from '../contexts/LyricsContext';
import { usePlayer } from '../contexts/PlayerContext';

export function LyricsPanel() {
  const { lyrics, activeLineIndex } = useLyrics();
  const { position } = usePlayer();
  const containerRef = useRef<HTMLDivElement>(null);
  const lineRefs = useRef<(HTMLDivElement | null)[]>([]);

  // Auto-scroll to active line
  useEffect(() => {
    if (activeLineIndex >= 0 && lineRefs.current[activeLineIndex]) {
      lineRefs.current[activeLineIndex]?.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
      });
    }
  }, [activeLineIndex]);

  if (lyrics.length === 0) {
    return (
      <div className="lyrics-panel">
        <div className="lyrics-header">
          <h3>Lyrics</h3>
        </div>
        <div className="lyrics-empty">
          No lyrics available. Place a .lrc file next to the audio file.
        </div>
      </div>
    );
  }

  return (
    <div className="lyrics-panel">
      <div className="lyrics-header">
        <h3>Lyrics</h3>
      </div>
      <div className="lyrics-content" ref={containerRef}>
        {lyrics.map((line, index) => (
          <div
            key={index}
            ref={(el) => (lineRefs.current[index] = el)}
            className={`lyric-line ${index === activeLineIndex ? 'active' : ''}`}
          >
            {line.text}
          </div>
        ))}
      </div>
    </div>
  );
}
