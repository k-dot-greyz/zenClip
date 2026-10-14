import { defineConfig } from 'astro/config';

export default defineConfig({
  security: {
    csp: {
      algorithm: 'SHA-256',
      directives: [
        "default-src 'self'",
        "img-src 'self' blob: data:",
        "media-src 'self' blob:",
        "connect-src 'self'",
        "worker-src 'self' blob:",
        "base-uri 'self'",
        "form-action 'self'",
        "object-src 'none'",
      ],
      scriptDirective: {
        resources: ["'self'", "'wasm-unsafe-eval'"],
      },
    },
  },
  vite: {
    optimizeDeps: {
      exclude: ['zenclip-wasm'],
    },
    build: {
      rollupOptions: {
        external: [],
      },
    },
  },
});
