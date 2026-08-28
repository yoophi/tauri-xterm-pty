import { Unicode11Addon } from "@xterm/addon-unicode11";
import { Terminal } from "@xterm/xterm";
import { describe, expect, it } from "vitest";

import { createTerminalOptions } from "@/features/terminal-session/model/terminal-options";

describe("terminal options", () => {
  it("allow the Unicode 11 addon to activate", () => {
    const terminal = new Terminal(createTerminalOptions());

    expect(() => terminal.loadAddon(new Unicode11Addon())).not.toThrow();
    expect(() => {
      terminal.unicode.activeVersion = "11";
    }).not.toThrow();
    expect(terminal.unicode.activeVersion).toBe("11");

    terminal.dispose();
  });
});
