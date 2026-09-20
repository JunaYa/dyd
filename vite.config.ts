import { cpSync } from 'node:fs'
import path from 'node:path'

import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

const host = process.env.TAURI_DEV_HOST

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    react(),
    {
      name: 'excalidraw-fonts',
      buildStart() {
        cpSync(
          path.resolve(import.meta.dirname, 'node_modules/@excalidraw/excalidraw/dist/prod/fonts'),
          path.resolve(import.meta.dirname, 'public/excalidraw/fonts'),
          { recursive: true },
        )
      },
    },
  ],

  resolve: {
    alias: {
      '~': path.resolve(import.meta.dirname, 'src'),
    },
  },

  define: {
    'process.env.IS_PREACT': JSON.stringify('true'),
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // The desktop launcher passes the actual listening URL to Tauri.
  server: {
    port: 1420,
    strictPort: false,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}))
