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
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ['**/src-tauri/**'],
    },
  },
}))
