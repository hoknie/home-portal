import "@testing-library/jest-dom/vitest";

import { cleanup } from "@testing-library/react";
import { afterEach, beforeEach, vi } from "vitest";

const originalError = console.error;
let unexpectedErrors: unknown[][] = [];

beforeEach(() => {
  unexpectedErrors = [];
  console.error = (...args: unknown[]) => {
    unexpectedErrors.push(args);
  };
});

afterEach(() => {
  cleanup();
  console.error = originalError;
  if (unexpectedErrors.length > 0) {
    const first = unexpectedErrors[0].map((part) => (part instanceof Error ? part.message : String(part))).join(" ");
    throw new Error(`console.error was called ${unexpectedErrors.length} time(s): ${first}`);
  }
});

if (!window.matchMedia) {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
}

class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}

if (!("ResizeObserver" in globalThis)) {
  Object.defineProperty(globalThis, "ResizeObserver", { writable: true, value: ResizeObserverStub });
}

if (!Element.prototype.hasPointerCapture) {
  Element.prototype.hasPointerCapture = () => false;
  Element.prototype.releasePointerCapture = () => {};
  Element.prototype.scrollIntoView = () => {};
}
