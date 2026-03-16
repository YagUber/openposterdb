import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'
import tailwindcss from '@tailwindcss/vite'
import placeholdersPlugin from './vite-plugin-placeholders'
import optimizeExamplesPlugin from './vite-plugin-optimize-examples'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    placeholdersPlugin(),
    optimizeExamplesPlugin(),
    vue(),
    vueDevTools(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    },
  },
  server: {
    proxy: {
      '/api': process.env.E2E_BACKEND_URL || 'http://localhost:3000',
      // Must match FREE_API_KEY in src/lib/constants.ts
      '/t0-free-rpdb': process.env.E2E_BACKEND_URL || 'http://localhost:3000',
    },
  },
})
