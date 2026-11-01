import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// The dev rig serves the same bundle the desktop binary embeds and the
// phone's WebView loads. There is one bundle; the transport decides where
// the data comes from.
export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    sourcemap: false,
    chunkSizeWarningLimit: 900,
  },
  server: {
    port: 5173,
    strictPort: true,
  },
})
