import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '../lib/invoke'

export const usePlaylistsStore = defineStore('playlists', () => {
  const playlists = ref<api.Playlist[]>([])

  async function load() {
    try {
      playlists.value = await api.getPlaylists()
    } catch (e) {
      console.error('Failed to load playlists', e)
    }
  }

  async function create(name: string): Promise<api.Playlist | null> {
    try {
      const p = await api.createPlaylist(name)
      await load()
      return p
    } catch (e) {
      console.error('Failed to create playlist', e)
      return null
    }
  }

  async function remove(id: string) {
    try {
      await api.deletePlaylist(id)
      await load()
    } catch (e) {
      console.error('Failed to delete playlist', e)
    }
  }

  async function rename(id: string, name: string) {
    try {
      await api.renamePlaylist(id, name)
      await load()
    } catch (e) {
      console.error('Failed to rename playlist', e)
    }
  }

  return { playlists, load, create, remove, rename }
})
