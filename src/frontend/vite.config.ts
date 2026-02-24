import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';
import path from 'path';

export default defineConfig({
  plugins: [solid()],
  resolve: {
    alias: {
      '~': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
    cssMinify: true,
    reportCompressedSize: false, // Faster builds
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor': ['solid-js', '@solidjs/router'],
          'tanstack': ['@tanstack/solid-query'],
          'kobalte': ['@kobalte/core'],
        },
        compact: true,
      },
      treeshake: {
        preset: 'recommended',
        moduleSideEffects: false,
      },
    },
  },
});
