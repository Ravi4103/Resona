import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '../lib/invoke'

export const useFavoritesStore = defineStore('favorites', () => {
  const favoriteIds = ref<Set<string>>(new Set())

  async function load() {
    try {
      const ids = await api.getFavoriteIds()
      favoriteIds.value = new Set(ids)
    } catch (e) {
      console.error('Failed to load favorites', e)
    }
  }

  function isFavorite(id: string): boolean {
    return favoriteIds.value.has(id)
  }

  async function toggle(id: string) {
    try {
      const nowFav = await api.toggleFavorite(id)
      if (nowFav) {
        favoriteIds.value.add(id)
      } else {
        favoriteIds.value.delete(id)
      }
      return nowFav
    } catch (e) {
      console.error('Failed to toggle favorite', e)
      return false
    }
  }

  return { favoriteIds, load, isFavorite, toggle }
})
