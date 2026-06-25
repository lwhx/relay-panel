// v0.4.10: global test setup. Adds jest-dom matchers (toBeInTheDocument etc.)
// to Vitest's expect so component assertions read naturally. Loaded once per
// test run via vitest config setupFiles.
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

// jsdom doesn't implement ResizeObserver or matchMedia, but antd components
// (Table, Collapse, Progress) reference them on mount. Provide minimal stubs so
// component-rendering tests don't crash. Harmless for non-antd tests.
if (!globalThis.ResizeObserver) {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  };
}

if (!window.matchMedia) {
  window.matchMedia = (query: string) =>
    ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    }) as unknown as MediaQueryList;
}

// Unmount rendered components between tests so each test starts from a clean
// DOM (otherwise multiple StateProbe instances accumulate and getByTestId finds
// duplicates). @testing-library/react auto-cleans when globals are enabled; we
// do it explicitly since vitest runs without globals here.
afterEach(() => {
  cleanup();
});
