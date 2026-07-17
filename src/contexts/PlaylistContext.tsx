import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Playlist, PlaylistTrack } from '../types';

interface PlaylistContextType {
  playlists: Playlist[];
  currentPlaylistId: number | null;
  playlistTracks: PlaylistTrack[];
  loadPlaylists: () => Promise<void>;
  createPlaylist: (name: string) => Promise<number>;
  renamePlaylist: (id: number, name: string) => Promise<void>;
  deletePlaylist: (id: number) => Promise<void>;
  selectPlaylist: (id: number) => Promise<void>;
  addToPlaylist: (playlistId: number, trackIds: string[]) => Promise<void>;
  removeFromPlaylist: (playlistId: number, trackIds: string[]) => Promise<void>;
  reorderPlaylist: (playlistId: number, order: string[]) => Promise<void>;
}

const PlaylistContext = createContext<PlaylistContextType | undefined>(undefined);

export function PlaylistProvider({ children }: { children: ReactNode }) {
  const [playlists, setPlaylists] = useState<Playlist[]>([]);
  const [currentPlaylistId, setCurrentPlaylistId] = useState<number | null>(null);
  const [playlistTracks, setPlaylistTracks] = useState<PlaylistTrack[]>([]);

  const loadPlaylists = useCallback(async () => {
    const list = await invoke<Playlist[]>('get_playlists');
    setPlaylists(list);
  }, []);

  const createPlaylist = useCallback(async (name: string) => {
    const id = await invoke<number>('create_playlist', { name });
    await loadPlaylists();
    return id;
  }, [loadPlaylists]);

  const renamePlaylist = useCallback(async (id: number, name: string) => {
    await invoke('rename_playlist', { id, name });
    await loadPlaylists();
  }, [loadPlaylists]);

  const deletePlaylist = useCallback(async (id: number) => {
    await invoke('delete_playlist', { id });
    if (currentPlaylistId === id) {
      setCurrentPlaylistId(null);
      setPlaylistTracks([]);
    }
    await loadPlaylists();
  }, [currentPlaylistId, loadPlaylists]);

  const selectPlaylist = useCallback(async (id: number) => {
    setCurrentPlaylistId(id);
    const tracks = await invoke<PlaylistTrack[]>('get_playlist_tracks', { playlistId: id });
    setPlaylistTracks(tracks);
  }, []);

  const addToPlaylist = useCallback(async (playlistId: number, trackIds: string[]) => {
    await invoke('add_to_playlist', { playlistId, trackIds });
    if (currentPlaylistId === playlistId) {
      await selectPlaylist(playlistId);
    }
  }, [currentPlaylistId, selectPlaylist]);

  const removeFromPlaylist = useCallback(async (playlistId: number, trackIds: string[]) => {
    await invoke('remove_from_playlist', { playlistId, trackIds });
    if (currentPlaylistId === playlistId) {
      await selectPlaylist(playlistId);
    }
  }, [currentPlaylistId, selectPlaylist]);

  const reorderPlaylist = useCallback(async (playlistId: number, order: string[]) => {
    await invoke('reorder_playlist', { playlistId, order });
    if (currentPlaylistId === playlistId) {
      await selectPlaylist(playlistId);
    }
  }, [currentPlaylistId, selectPlaylist]);

  useEffect(() => {
    loadPlaylists();
  }, [loadPlaylists]);

  return (
    <PlaylistContext.Provider
      value={{
        playlists,
        currentPlaylistId,
        playlistTracks,
        loadPlaylists,
        createPlaylist,
        renamePlaylist,
        deletePlaylist,
        selectPlaylist,
        addToPlaylist,
        removeFromPlaylist,
        reorderPlaylist,
      }}
    >
      {children}
    </PlaylistContext.Provider>
  );
}

export function usePlaylist() {
  const context = useContext(PlaylistContext);
  if (!context) {
    throw new Error('usePlaylist must be used within a PlaylistProvider');
  }
  return context;
}
