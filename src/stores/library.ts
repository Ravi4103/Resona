import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '../lib/invoke'

export const useLibraryStore = defineStore('library', () => {
  const tracks = ref<api.Track[]>([])
  const playlists = ref<api.Playlist[]>([])
  const loading = ref(false)

  async function loadAll() {
    loading.value = true
    try {
      tracks.value = await api.getTracks()
      playlists.value = await api.getPlaylists()
    } catch (e) {
      console.error('Failed to load library', e)
    } finally {
      loading.value = false
    }
  }

  async function loadPlaylists() {
    try {
      playlists.value = await api.getPlaylists()
    } catch (e) {
      console.error('Failed to load playlists', e)
    }
  }

  async function createPlaylist(name: string): Promise<api.Playlist | null> {
    try {
      const p = await api.createPlaylist(name)
      await loadPlaylists()
      return p
    } catch (e) {
      console.error('Failed to create playlist', e)
      return null
    }
  }

  async function deletePlaylist(id: string) {
    try {
      await api.deletePlaylist(id)
      await loadPlaylists()
    } catch (e) {
      console.error('Failed to delete playlist', e)
    }
  }

  async function deleteTracks(ids: string[], deleteFiles: boolean) {
    try {
      await api.deleteTracks(ids, deleteFiles)
      tracks.value = tracks.value.filter(t => !ids.includes(t.id))
    } catch (e) {
      console.error('Failed to delete tracks', e)
    }
  }

  async function renamePlaylist(id: string, name: string) {
    try {
      await api.renamePlaylist(id, name)
      await loadPlaylists()
    } catch (e) {
      console.error('Failed to rename playlist', e)
    }
  }

  async function importPlaylist(filePath: string, name: string): Promise<api.Playlist | null> {
    try {
      const p = await api.importPlaylistM3U(filePath, name)
      await loadPlaylists()
      return p
    } catch (e) {
      console.error('Failed to import playlist', e)
      return null
    }
  }

  return {
    tracks, playlists, loading, loadAll, loadPlaylists, createPlaylist, deletePlaylist,
    deleteTracks, renamePlaylist, importPlaylist,
  }
})
