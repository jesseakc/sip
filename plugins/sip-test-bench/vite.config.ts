import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// The SIP API serves this plugin's built bundle from plugins/<id>/dist at
//   GET /api/v1/ui/plugins/sip-test-bench/app/<path>
// and the SIP shell mounts it in an iframe. The document is therefore loaded
// from a deep path, so every asset URL must be RELATIVE — `base: './'` is
// load-bearing here, not cosmetic: an absolute '/assets/...' URL would resolve
// against the API origin root and 404.
export default defineConfig({
  base: './',
  plugins: [react()],
  build: {
    // Relative to this file's directory (Vite's `root`), i.e.
    //   plugins/sip-test-bench/dist
    // which is the layout the SIP API serves:
    //   GET /api/v1/ui/plugins/sip-test-bench/app/<path> -> plugins/sip-test-bench/dist/<path>
    // Note: '../dist' would resolve one level too high (plugins/dist) because
    // outDir is relative to the project root, not to dist/.
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2020',
    // Keep the asset layout predictable for the host's static file server.
    assetsDir: 'assets',
    sourcemap: false,
  },
  server: {
    port: 5173,
    strictPort: false,
  },
  preview: {
    port: 4173,
  },
});
