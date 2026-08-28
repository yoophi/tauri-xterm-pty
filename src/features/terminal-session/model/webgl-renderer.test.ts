import type { WebglAddon } from "@xterm/addon-webgl";
import type { Terminal } from "@xterm/xterm";
import { afterEach, describe, expect, it, vi } from "vitest";

import { enableWebglRenderer } from "@/features/terminal-session/model/webgl-renderer";

type TerminalAddonHost = Pick<Terminal, "loadAddon">;

function createAddonDouble() {
  let contextLossListener: (() => void) | undefined;
  const dispose = vi.fn();
  const addon = {
    dispose,
    onContextLoss: vi.fn((listener: () => void) => {
      contextLossListener = listener;
      return { dispose: vi.fn() };
    }),
  } as unknown as WebglAddon;

  return {
    addon,
    dispose,
    loseContext: () => contextLossListener?.(),
  };
}

describe("WebGL terminal renderer", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("loads the WebGL addon", () => {
    const terminal = { loadAddon: vi.fn() } as TerminalAddonHost;
    const { addon } = createAddonDouble();

    expect(enableWebglRenderer(terminal, () => addon)).toBe(true);
    expect(terminal.loadAddon).toHaveBeenCalledWith(addon);
  });

  it("disposes the WebGL addon when its context is lost", () => {
    vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const terminal = { loadAddon: vi.fn() } as TerminalAddonHost;
    const { addon, dispose, loseContext } = createAddonDouble();

    enableWebglRenderer(terminal, () => addon);
    loseContext();

    expect(dispose).toHaveBeenCalledOnce();
  });

  it("keeps the DOM renderer when WebGL initialization fails", () => {
    vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const terminal = {
      loadAddon: vi.fn(() => {
        throw new Error("WebGL2 is unavailable");
      }),
    } as TerminalAddonHost;
    const { addon, dispose } = createAddonDouble();

    expect(enableWebglRenderer(terminal, () => addon)).toBe(false);
    expect(dispose).toHaveBeenCalledOnce();
  });
});
