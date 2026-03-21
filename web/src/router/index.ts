import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import TracksView from '@/views/TracksView.vue'

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/tracks' },
  { path: '/tracks', name: 'tracks', component: TracksView },
  {
    path: '/albums',
    name: 'albums',
    component: () => import('@/views/AlbumsView.vue'),
  },
  {
    path: '/artists',
    name: 'artists',
    component: () => import('@/views/ArtistsView.vue'),
  },
  {
    path: '/playlists',
    name: 'playlists',
    component: () => import('@/views/PlaylistsView.vue'),
  },
  {
    path: '/stats',
    name: 'stats',
    component: () => import('@/views/StatsView.vue'),
  },
]

const router = createRouter({
  history: createWebHashHistory('/web/'),
  routes,
})

export default router
