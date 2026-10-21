import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

const isTauri = process.env.TAURI_ENV_PLATFORM !== undefined;

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  base: isTauri ? '/' : (process.env.VITE_BASE_PATH || '/'),
  server: {
    port: 5173,
    strictPort: true, // Prevent Vite from auto-switching to 5174 when Tauri expects 5173
    host: '127.0.0.1', // Force IPv4 loopback to prevent Windows WebView2 IPv6 resolution failures
    watch: {
      ignored: ['**/src-tauri/**'],
    },
  },
});
