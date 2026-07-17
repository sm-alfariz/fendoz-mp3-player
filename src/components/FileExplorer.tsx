import { useState } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { usePlayer } from '../contexts/PlayerContext';
import { usePlaylist } from '../contexts/PlaylistContext';
import { Track } from '../types';

export function FileExplorer() {
  const { setTracks, play } = usePlayer();
  const { currentPlaylistId, addToPlaylist } = usePlaylist();
  const [currentPath, setCurrentPath] = useState<string>('');
  const [loading, setLoading] = useState(false);

  const handleOpenFolder = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Music Folder',
    });

    if (selected) {
      setCurrentPath(selected as string);
      setLoading(true);

      try {
        const scannedTracks = await invoke<Track[]>('scan_directory', {
          path: selected as string,
        });
        setTracks(scannedTracks);
      } catch (error) {
        console.error('Error scanning directory:', error);
      } finally {
        setLoading(false);
      }
    }
  };

  const handleAddToPlaylist = async (track: Track) => {
    if (currentPlaylistId) {
      await addToPlaylist(currentPlaylistId, [track.id]);
    }
  };

  const { tracks } = usePlayer();

  return (
    <div className="file-explorer">
      <div className="explorer-header">
        <h3>Library</h3>
        <button className="open-folder-btn" onClick={handleOpenFolder}>
          📁 Open Folder
        </button>
      </div>

      {currentPath && (
        <div className="current-path">
          <span className="path-label">Location:</span>
          <span className="path-value">{currentPath}</span>
        </div>
      )}

      {loading ? (
        <div className="loading">Scanning files...</div>
      ) : (
        <div className="track-list">
          {tracks.length === 0 ? (
            <div className="empty-library">
              No tracks loaded. Click "Open Folder" to scan for music files.
            </div>
          ) : (
            tracks.map((track) => (
              <div key={track.id} className="track-item">
                <div className="track-info" onClick={() => play(track)}>
                  <div className="track-title">{track.title}</div>
                  <div className="track-artist">
                    {track.artist} • {track.album}
                  </div>
                </div>
                <button
                  className="add-to-playlist-btn"
                  onClick={() => handleAddToPlaylist(track)}
                  disabled={!currentPlaylistId}
                  title={currentPlaylistId ? 'Add to playlist' : 'Select a playlist first'}
                >
                  +
                </button>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
