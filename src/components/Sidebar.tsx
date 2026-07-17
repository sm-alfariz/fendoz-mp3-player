import React, { useState } from 'react';
import { usePlaylist } from '../contexts/PlaylistContext';

export function Sidebar() {
  const {
    playlists,
    currentPlaylistId,
    selectPlaylist,
    createPlaylist,
    renamePlaylist,
    deletePlaylist,
  } = usePlaylist();

  const [newPlaylistName, setNewPlaylistName] = useState('');
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editingName, setEditingName] = useState('');

  const handleCreatePlaylist = async () => {
    if (newPlaylistName.trim()) {
      await createPlaylist(newPlaylistName.trim());
      setNewPlaylistName('');
    }
  };

  const handleRename = async (id: number) => {
    if (editingName.trim()) {
      await renamePlaylist(id, editingName.trim());
      setEditingId(null);
      setEditingName('');
    }
  };

  const handleDelete = async (id: number) => {
    if (window.confirm('Delete this playlist?')) {
      await deletePlaylist(id);
    }
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h2>Playlists</h2>
        <button className="add-playlist-btn" onClick={() => setNewPlaylistName('')}>
          +
        </button>
      </div>

      <div className="new-playlist-form">
        <input
          type="text"
          placeholder="New playlist name..."
          value={newPlaylistName}
          onChange={(e) => setNewPlaylistName(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleCreatePlaylist()}
        />
        <button onClick={handleCreatePlaylist} disabled={!newPlaylistName.trim()}>
          Create
        </button>
      </div>

      <div className="playlist-list">
        {playlists.map((playlist) => (
          <div
            key={playlist.id}
            className={`playlist-item ${currentPlaylistId === playlist.id ? 'active' : ''}`}
            onClick={() => selectPlaylist(playlist.id)}
          >
            {editingId === playlist.id ? (
              <div className="edit-playlist">
                <input
                  type="text"
                  value={editingName}
                  onChange={(e) => setEditingName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleRename(playlist.id);
                    if (e.key === 'Escape') setEditingId(null);
                  }}
                  onBlur={() => handleRename(playlist.id)}
                  autoFocus
                />
              </div>
            ) : (
              <>
                <span className="playlist-name">{playlist.name}</span>
                <div className="playlist-actions">
                  <button
                    className="action-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      setEditingId(playlist.id);
                      setEditingName(playlist.name);
                    }}
                  >
                    ✏️
                  </button>
                  <button
                    className="action-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDelete(playlist.id);
                    }}
                  >
                    🗑️
                  </button>
                </div>
              </>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
