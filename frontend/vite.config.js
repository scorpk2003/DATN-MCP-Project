import process from "node:process"
import { defineConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    babel({ presets: [reactCompilerPreset()] })
  ],
  server: {
    proxy: {
      "/api/agent-gateway": {
        target: process.env.VITE_AGENT_GATEWAY_PROXY_TARGET || "http://localhost:4000",
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api\/agent-gateway/, ""),
      },
    },
  },
})
