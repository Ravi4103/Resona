import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  { path: '/', name: 'home', component: () => import('../views/HomeView.vue') },
  { path: '/albums', name: 'albums', component: () => import('../views/AlbumsView.vue') },
  { path: '/albums/:id', name: 'album-detail', component: () => import('../views/AlbumDetailView.vue') },
  {
    path: '/artists',
    name: 'artists',
    component: () => import('../views/ArtistsView.vue'),
    children: [
      { path: ':id', name: 'artist-detail', component: () => import('../views/ArtistDetailView.vue') },
    ]
  },
  { path: '/tracks', name: 'tracks', component: () => import('../views/TracksView.vue') },
  {
    path: '/genres',
    name: 'genres',
    component: () => import('../views/GenresView.vue'),
    children: [
      { path: ':id', name: 'genre-detail', component: () => import('../views/GenreDetailView.vue') },
    ]
  },
  {
    path: '/composers',
    name: 'composers',
    component: () => import('../views/ComposersView.vue'),
    children: [
      { path: ':id', name: 'composer-detail', component: () => import('../views/ComposerDetailView.vue') },
    ]
  },
  { path: '/mini-player', name: 'mini-player', component: () => import('../views/MiniPlayerWindowView.vue') },
  { path: '/favorites', redirect: '/playlists/favorites' },
  { path: '/search', name: 'search', component: () => import('../views/SearchView.vue') },
  { path: '/radio', name: 'radio', component: () => import('../views/RadioView.vue') },
  { path: '/playlists', name: 'playlists', component: () => import('../views/PlaylistListView.vue') },
  { path: '/playlists/:id', name: 'playlist-detail', component: () => import('../views/PlaylistDetailView.vue') },
  { path: '/settings/:category?', name: 'settings', component: () => import('../views/SettingsView.vue'), props: true },
]

const router = createRouter({ history: createWebHashHistory(), routes })
export default router
