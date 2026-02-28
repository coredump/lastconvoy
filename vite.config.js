import { defineConfig } from 'vite'

export default defineConfig({
  root: 'deploy',
  server: {
    port: 8080,
    open: true,
    headers: {
      'Cross-Origin-Opener-Policy': 'same-origin',
      'Cross-Origin-Embedder-Policy': 'require-corp',
    },
  },
})
