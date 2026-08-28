import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  createRoot: vi.fn(() => ({ render: vi.fn() })),
}));

vi.mock("react-dom/client", () => ({
  default: { createRoot: mocks.createRoot },
}));

describe("application bootstrap", () => {
  beforeEach(() => {
    document.body.innerHTML = '<div id="root"></div>';
  });

  it("mounts the application without waiting for terminal font loading", async () => {
    await import("@/main");

    expect(mocks.createRoot).toHaveBeenCalledOnce();
  });
});
