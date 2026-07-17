import React from 'react';
import { usePlayer } from '../contexts/PlayerContext';
import { PlaybackMode } from '../types';

export function PlayerBar() {
  const {
    currentTrack,
    position,
    duration,
    state,
    mode,
    volume,
    play,
    pause,
    resume,
    stop,
    seek,
    setVolume,
    nextTrack,
    prevTrack,
    setMode,
  } = usePlayer();

  const formatTime = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  const handleSeek = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newTime = parseInt(e.target.value);
    seek(newTime);
  };

  const handleVolumeChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newVolume = parseFloat(e.target.value);
    setVolume(newVolume);
  };

  const togglePlayPause = () => {
    if (state === 'playing') {
      pause();
    } else {
      resume();
    }
  };

  const cycleMode = () => {
    const modes: PlaybackMode[] = ['loop_all', 'loop_single', 'shuffle'];
    const currentIndex = modes.indexOf(mode);
    const nextIndex = (currentIndex + 1) % modes.length;
    setMode(modes[nextIndex]);
  };

  const getModeIcon = () => {
    switch (mode) {
      case 'loop_single':
        return '🔂';
      case 'shuffle':
        return '🔀';
      default:
        return '🔁';
    }
  };

  return (
    <div className="player-bar">
      <div className="track-info">
        {currentTrack ? (
          <>
            <div className="track-title">{currentTrack.title}</div>
            <div className="track-artist">{currentTrack.artist}</div>
          </>
        ) : (
          <div className="no-track">No track selected</div>
        )}
      </div>

      <div className="playback-controls">
        <button className="control-btn" onClick={prevTrack} title="Previous">
          ⏮
        </button>
        <button className="control-btn play-btn" onClick={togglePlayPause}>
          {state === 'playing' ? '⏸' : '▶'}
        </button>
        <button className="control-btn" onClick={nextTrack} title="Next">
          ⏭
        </button>
        <button className="control-btn mode-btn" onClick={cycleMode} title={`Mode: ${mode}`}>
          {getModeIcon()}
        </button>
      </div>

      <div className="seek-bar">
        <span className="time">{formatTime(position)}</span>
        <input
          type="range"
          min="0"
          max={duration || 100}
          value={position}
          onChange={handleSeek}
          className="seek-slider"
        />
        <span className="time">{formatTime(duration)}</span>
      </div>

      <div className="volume-control">
        <span className="volume-icon">🔊</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={volume}
          onChange={handleVolumeChange}
          className="volume-slider"
        />
      </div>
    </div>
  );
}
