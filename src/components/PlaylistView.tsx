import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { usePlayer } from '../contexts/PlayerContext';
import { usePlaylist } from '../contexts/PlaylistContext';
import { Track } from '../types';

export function PlaylistView() {
  const { currentPlaylistId, playlistTracks, removeFromPlaylist, reorderPlaylist } = usePlaylist();
  const { tracks, play, currentTrack } = usePlayer();
  const [playlistTrackDetails, setPlaylistTrackDetails] = useState<Track[]>([]);
  const [dragIndex, setDragIndex] = useState<number | null>(null);

  // Load track details for playlist tracks
  useEffect(() => {
    const loadTrackDetails = async () => {
      const details = playlistTracks
        .map((pt) => tracks.find((t) => t.id === pt.track_id))
        .filter(Boolean) as Track[];
      setPlaylistTrackDetails(details);
    };

    loadTrackDetails();
  }, [playlistTracks, tracks]);

  const handleDragStart = (index: number) => {
    setDragIndex(index);
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    if (dragIndex === null || dragIndex === index) return;

    const newTracks = [...playlistTrackDetails];
    const draggedTrack = newTracks[dragIndex];
    newTracks.splice(dragIndex, 1);
    newTracks.splice(index, 0, draggedTrack);

    setPlaylistTrackDetails(newTracks);
    setDragIndex(index);
  };

  const handleDragEnd = async () => {
    setDragIndex(null);
    if (currentPlaylistId && playlistTrackDetails.length > 0) {
      const newOrder = playlistTrackDetails.map((t) => t.id);
      await reorderPlaylist(currentPlaylistId, newOrder);
    }
  };

  const handleRemoveFromPlaylist = async (trackId: string) => {
    if (currentPlaylistId) {
      await removeFromPlaylist(currentPlaylistId, [trackId]);
    }
  };

  const formatDuration = (ms: number) => {
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}:${seconds.toString().padStart(2, '0')}`;
  };

  const totalDuration = playlistTrackDetails.reduce((sum, t) => sum + t.duration_ms, 0);

  if (!currentPlaylistId) {
    return (
      <div className="playlist-view">
        <div className="playlist-header">
          <h2>Select a Playlist</h2>
        </div>
        <div className="playlist-empty">
          Choose a playlist from the sidebar or create a new one.
        </div>
      </div>
    );
  }

  return (
    <div className="playlist-view">
      <div className="playlist-header">
        <h2>Playlist</h2>
        <div className="playlist-stats">
          <span>{playlistTrackDetails.length} tracks</span>
          <span>•</span>
          <span>{formatDuration(totalDuration)}</span>
        </div>
      </div>

      <div className="track-list">
        {playlistTrackDetails.length === 0 ? (
          <div className="playlist-empty">
            No tracks in this playlist. Add tracks from the file explorer.
          </div>
        ) : (
          playlistTrackDetails.map((track, index) => (
            <div
              key={track.id}
              className={`track-item ${currentTrack?.id === track.id ? 'playing' : ''}`}
              draggable
              onDragStart={() => handleDragStart(index)}
              onDragOver={(e) => handleDragOver(e, index)}
              onDragEnd={handleDragEnd}
            >
              <div className="track-number">{index + 1}</div>
              <div className="track-info" onClick={() => play(track)}>
                <div className="track-title">{track.title}</div>
                <div className="track-artist">{track.artist}</div>
              </div>
              <div className="track-duration">{formatDuration(track.duration_ms)}</div>
              <button
                className="remove-btn"
                onClick={() => handleRemoveFromPlaylist(track.id)}
              >
                ✕
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
