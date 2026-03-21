import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  base: '/web/',
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  build: {
    outDir: '../static',
    emptyOutDir: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'naive-ui': ['naive-ui'],
          vue: ['vue', 'pinia'],
        },
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/tracks': 'http://localhost:3000',
      '/stream': 'http://localhost:3000',
      '/cover': 'http://localhost:3000',
      '/lyrics': 'http://localhost:3000',
      '/albums': 'http://localhost:3000',
      '/artists': 'http://localhost:3000',
      '/playlists': 'http://localhost:3000',
      '/stats': 'http://localhost:3000',
    },
  },
})
