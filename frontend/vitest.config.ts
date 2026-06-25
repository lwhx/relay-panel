import { defineConfig, mergeConfig } from 'vitest/config';
import viteConfig from './vite.config';

// v0.4.10: separate vitest config. vite.config.ts stays a pure vite config
// (so `vite build` and tsc see no unknown `test` field); this file adds the
// test settings on top. mergeConfig keeps the two in sync automatically.
export default mergeConfig(
  viteConfig,
  defineConfig({
    test: {
      // Component tests need a DOM. Pure-logic tests (e.g. version.test) are
      // unaffected — jsdom provides a window they simply don't touch.
      environment: 'jsdom',
      setupFiles: ['./src/test/setup.ts'],
      exclude: ['node_modules', 'dist'],
    },
  })
);
