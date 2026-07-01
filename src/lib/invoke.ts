import { invoke } from '@tauri-apps/api/core'

export interface Track {
  id: string
  file_path: string
  title: string
  artist: string
  album: string
  album_artist: string
  composer: string
  track_number: number
  disc_number: number
  year: number
  genre: string
  raw_genre_names: string
  duration: number
  sample_rate: number
  bit_depth: number
  file_size: number
  file_format: string
  has_artwork: boolean
  replaygain_track_gain: number
  replaygain_album_gain: number
  cue_parent_id: string | null
  cue_offset: number
  artwork_key: string | null
}

export interface Playlist {
  id: string
  name: string
  description: string
  created_at: string
}

export interface ScanResult {
  added: number
  updated: number
  removed: number
}

export interface Genre {
  id: string
  name: string
  normalization_key: string
  track_count: number
}

export interface SearchAllResults {
  tracks: Track[]
  albums: { name: string; artist: string; year: number; track_count: number; first_track_id: string }[]
  artists: string[]
  genres: string[]
  composers: string[]
}

export interface LyricsResult {
  title: string
  artist: string
  album: string
  lyrics: string
  source: string
}

export interface LyricRow {
  track_id: string
  content: string
  source: string
  meta_content: string
  meta_source: string
}

export const saveLyrics = (trackId: string, content: string, source: string) => invoke<void>('save_lyrics', { trackId, content, source })

export const getTrackLyrics = (trackId: string) => invoke<LyricRow | null>('get_track_lyrics', { trackId })

export const batchFetchLyrics = (folderPath?: string) => invoke<string>('batch_fetch_lyrics', { folderPath })

export const getTracks = () => invoke<Track[]>('get_tracks')
export const getTrack = (id: string) => invoke<Track | null>('get_track', { id })
export const searchTracks = (query: string) => invoke<Track[]>('search_tracks', { query })
export const searchAll = (query: string) => invoke<SearchAllResults>('search_all', { query })
export const scanFolder = (path: string) => invoke<ScanResult>('scan_folder', { path })
export const getArtwork = (id: string) => invoke<string>('get_artwork', { id })
export const seek = (posSecs: number) => invoke<void>('seek', { posSecs })
export const playTrack = (id: string) => invoke<void>('play_track', { id })
export const togglePlay = () => invoke<boolean>('toggle_play')
export const setVolume = (vol: number) => invoke<void>('set_volume', { vol })
export const setEqBand = (index: number, gainDb: number) => invoke<void>('set_eq_band', { index, gainDb })
export const setEqEnabled = (enabled: boolean) => invoke<void>('set_eq_enabled', { enabled })
export const pingAudio = () => invoke<void>('ping_audio')

export const getPosition = () => invoke<number>('get_position')
export const getPlaylists = () => invoke<Playlist[]>('get_playlists')
export const createPlaylist = (name: string) => invoke<Playlist>('create_playlist', { name })
export const deletePlaylist = (id: string) => invoke<void>('delete_playlist', { id })
export const addToPlaylist = (playlistId: string, trackId: string) => invoke<void>('add_to_playlist', { playlistId, trackId })
export const addTracksToPlaylist = (playlistId: string, trackIds: string[]) => invoke<void>('add_tracks_to_playlist', { playlistId, trackIds })
export const removeFromPlaylist = (playlistId: string, trackId: string) => invoke<void>('remove_from_playlist', { playlistId, trackId })
export const getPlaylistTracks = (playlistId: string) => invoke<Track[]>('get_playlist_tracks', { playlistId })
export const getFavoriteIds = () => invoke<string[]>('get_favorite_ids')
export const getLyrics = (filePath: string) => invoke<string | null>('get_lyrics', { filePath })
export const toggleFavorite = (id: string) => invoke<boolean>('toggle_favorite', { id })
export const getLibraryFolders = () => invoke<string[]>('get_library_folders')
export const addLibraryFolder = (path: string) => invoke<void>('add_library_folder', { path })
export const addLibraryFolders = (paths: string[]) => invoke<ScanResult>('add_library_folders', { paths })
export const removeLibraryFolder = (path: string) => invoke<void>('remove_library_folder', { path })
export const deleteTracks = (ids: string[], deleteFiles: boolean) => invoke<void>('delete_tracks', { ids, deleteFiles })
export const recordPlay = (id: string) => invoke<void>('record_play', { id })
export const getRecentlyPlayedTracks = (limit: number) => invoke<Track[]>('get_recently_played_tracks', { limit })
export const getMostListenedTracks = (limit: number) => invoke<Track[]>('get_most_listened_tracks', { limit })
export const getLeastListenedTracks = (limit: number) => invoke<Track[]>('get_least_listened_tracks', { limit })
export const getAllGenres = () => invoke<Genre[]>('get_all_genres')
export const getTracksByGenreId = (genreId: string) => invoke<Track[]>('get_tracks_by_genre_id', { genreId })
export const showInExplorer = (id: string) => invoke<void>('show_in_explorer', { id })
export const findLyrics = (title: string, artist: string, album: string, save?: string) => invoke<LyricsResult[]>('find_lyrics', { title, artist, album, save })
export const updateTrackMetadata = (input: { id: string; title: string; artist: string; album: string; album_artist: string; genre: string; year: number; track_number: number; disc_number: number }) => invoke<void>('update_track_metadata', { input })
export const getPlaylistsForTrack = (trackId: string) => invoke<string[]>('get_playlists_for_track', { trackId })
export const renamePlaylist = (id: string, name: string) => invoke<void>('rename_playlist', { id, name })
export const exportPlaylistM3U = (id: string, outputPath: string) => invoke<void>('export_playlist_m3u', { id, outputPath })
export const importPlaylistM3U = (filePath: string, name: string) => invoke<Playlist>('import_playlist_m3u', { filePath, name })
export const getPlaylistTrackCount = (id: string) => invoke<number>('get_playlist_track_count', { id })
export const reorderPlaylistTracks = (playlistId: string, trackIds: string[]) => invoke<void>('reorder_playlist_tracks', { playlistId, trackIds })
export const getArtistArtwork = (artist: string, size?: string) => invoke<number[] | null>('get_artist_artwork', { artist, size })
export const getTrackArtwork = (id: string, size?: string) => invoke<number[] | null>('get_track_artwork', { id, size })
export const setArtwork = (id: string, imageB64: string) => invoke<void>('set_artwork', { id, imageB64 })
export const remoteStart = (port: number, password: string) => invoke<void>('remote_start', { port, password })
export const remoteStop = () => invoke<void>('remote_stop')
export const remoteStatus = () => invoke<boolean>('remote_status')
export const discordConnect = (clientId: string) => invoke<void>('discord_connect', { clientId })
export const discordDisconnect = () => invoke<void>('discord_disconnect')
export const updateDiscordPresence = (stateStr: string, details: string, startTime: number | null, artUrl: string | null) =>
  invoke<void>('update_discord_presence', { stateStr, details, startTime, artUrl })

export const preventSleep = () => invoke<void>('prevent_sleep')
export const allowSleep = () => invoke<void>('allow_sleep')
export const nowPlaying = (artist: string, track: string, album: string, duration: number) =>
  invoke<void>('now_playing', { artist, track, album, duration })
export const scrobble = (artist: string, track: string, album: string) =>
  invoke<void>('scrobble', { artist, track, album })

export interface RadioStation {
  id: string
  name: string
  url: string
  genre: string
  logo_url: string
  country: string
  language: string
  bitrate: number
}

export const getRadioStations = () => invoke<RadioStation[]>('get_radio_stations')
export const addRadioStation = (station: RadioStation) => invoke<void>('add_radio_station', { station })
export const deleteRadioStation = (id: string) => invoke<void>('delete_radio_station', { id })
export const updateRadioStation = (station: RadioStation) => invoke<void>('update_radio_station', { station })
export const playRadio = (url: string) => invoke<void>('play_radio', { url })
export const stopRadio = () => invoke<void>('stop_radio')
export const radioStatus = () => invoke<boolean>('radio_status')
export const closeMiniWindow = () => invoke<void>('close_mini_window')

// --- EQ Profiles ---
export interface EQProfile {
  id: string
  name: string
  is_default: boolean
  is_active: boolean
  bands: number[]
}

export const getEqProfiles = () => invoke<EQProfile[]>('get_eq_profiles')
export const getActiveEqProfile = () => invoke<EQProfile | null>('get_active_eq_profile')
export const createEqProfile = (name: string) => invoke<EQProfile>('create_eq_profile', { name })
export const renameEqProfile = (id: string, name: string) => invoke<void>('rename_eq_profile', { id, name })
export const deleteEqProfile = (id: string) => invoke<void>('delete_eq_profile', { id })
export const applyEqProfile = (id: string) => invoke<void>('apply_eq_profile', { id })
export const updateEqBand = (profileId: string, index: number, gainDb: number) => invoke<void>('update_eq_band', { profileId, index, gainDb })

export interface ThemeColors {
  vibrant: string
  muted: string
  dominant: string
}

export const getTrackPalette = (id: string) => invoke<ThemeColors | null>('get_track_palette', { id })

export interface PlayerState {
  track_id: string | null
  is_playing: boolean
  position: number
  volume: number
}

export const getPlayerState = () => invoke<PlayerState>('get_player_state')
