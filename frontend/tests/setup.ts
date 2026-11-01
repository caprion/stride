import '@testing-library/jest-dom/vitest'

/*
 * The browser bits jsdom does not have.
 *
 * The charts ask for a size; jsdom has no layout engine, so the observer
 * is stubbed with a size that is enough for the assertions that matter.
 */

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

globalThis.ResizeObserver = globalThis.ResizeObserver ?? (ResizeObserverStub as never)

if (!window.matchMedia) {
  window.matchMedia = ((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => false,
  })) as never
}

/** Every test runs against the synthetic fixture set. */
window.history.pushState({}, '', '/?fixtures=1')
