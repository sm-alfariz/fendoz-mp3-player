import { useEffect } from 'react';
import { usePlayer } from '../contexts/PlayerContext';
import { useLyrics } from '../contexts/LyricsContext';
import { Visualizer } from './Visualizer';
import { LyricsPanel } from './LyricsPanel';
import { Equalizer } from './Equalizer';

export function NowPlaying() {
  const player = usePlayer();
  const { loadLyrics, updateActiveLine } = useLyrics();

  useEffect(() => {
    if (player.currentTrack) {
      loadLyrics(player.currentTrack);
    }
  }, [player.currentTrack, loadLyrics]);

  useEffect(() => {
    updateActiveLine(player.position);
  }, [player.position, updateActiveLine]);

  return (
    <div className="now-playing">
      <div className="album-art">
        {player.currentTrack ? (
          <div className="art-placeholder">🎵</div>
        ) : (
          <div className="art-placeholder">🎶</div>
        )}
      </div>

      <div className="track-details">
        <h2 className="track-title">{player.currentTrack?.title || 'No Track'}</h2>
        <p className="track-artist">{player.currentTrack?.artist || 'Unknown Artist'}</p>
        <p className="track-album">{player.currentTrack?.album || 'Unknown Album'}</p>
      </div>

      <div className="visualizer-container">
        <Visualizer />
      </div>

      <div className="lyrics-container">
        <LyricsPanel />
      </div>

      <div className="eq-container">
        <Equalizer />
      </div>
    </div>
  );
}
