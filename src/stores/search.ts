import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as api from '../lib/invoke'

export const useSearchStore = defineStore('search', () => {
  const query = ref('')
  const results = ref<api.SearchAllResults>({ tracks: [], albums: [], artists: [], genres: [], composers: [] })
  const isSearching = ref(false)
  let abortController: AbortController | null = null
  let debounceTimer: ReturnType<typeof setTimeout> | null = null

  async function search(q: string) {
    query.value = q
    if (debounceTimer) clearTimeout(debounceTimer)
    if (abortController) abortController.abort()
    if (!q.trim()) {
      results.value = { tracks: [], albums: [], artists: [], genres: [], composers: [] }
      isSearching.value = false
      return
    }
    debounceTimer = setTimeout(async () => {
      isSearching.value = true
      abortController = new AbortController()
      try {
        results.value = await api.searchAll(q.trim())
      } catch {
        if (!abortController?.signal.aborted) {
          results.value = { tracks: [], albums: [], artists: [], genres: [], composers: [] }
        }
      }
      isSearching.value = false
    }, 300)
  }

  function clear() {
    query.value = ''
    results.value = { tracks: [], albums: [], artists: [], genres: [], composers: [] }
    isSearching.value = false
    if (debounceTimer) clearTimeout(debounceTimer)
  }

  return { query, results, isSearching, search, clear }
})
