import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    fs: {
      allow: ['../game-logic/pkg', '.']
    }
  },
  plugins: [react()],
})
