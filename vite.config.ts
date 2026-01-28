import { defineConfig } from 'vite';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 14206,
    strictPort: true,
    host: '127.0.0.1',
    hmr: {
      protocol: 'ws',
      host: '127.0.0.1',
      port: 14207,
    },
    watch: {
      // 3. tell vite to ignore watching `src` (Rust backend)
      ignored: [
        '**/src/**',
        '**/target/**',
        '**/node_modules/**',
        '**/.git/**',
        '**/data/**',
        '**/gen/**',
        '**/permissions/**',
        '**/scripts/**',
        '**/*.log',
        '**/*.csv',
        '**/*.parquet',
        '**/*.json',
        '**/*.py',
      ],
      // Reduce polling frequency to avoid excessive reloads
      interval: 1000,
      // Use efficient watching
      usePolling: false,
      awaitWriteFinish: {
        stabilityThreshold: 500,
        pollInterval: 100,
      },
    },
  },
  // 4. optimize bundle chunking for large dependencies
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Monaco Editor is large (~5.6 MB), split into separate chunk
          'monaco-editor': ['monaco-editor'],
          // Chart.js is moderate (~2 MB), split into separate chunk
          chart: ['chart.js'],
          // Tauri APIs are frequently used, keep together
          tauri: ['@tauri-apps/api', '@tauri-apps/plugin-dialog', '@tauri-apps/plugin-shell'],
        },
      },
    },
    // Monaco Editor is legitimately 5.6 MB (it's a full code editor like VS Code)
    // This is acceptable for a desktop app, suppress the warning
    chunkSizeWarningLimit: 6000,
    // Disable default emptyOutDir to avoid EPERM on Windows/OneDrive
    // We handle cleanup in scripts/clean.js
    emptyOutDir: false,
  },
}));
