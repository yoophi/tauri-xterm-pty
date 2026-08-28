import { describe, expect, it, vi } from "vitest";

import {
  loadTerminalFont,
  TERMINAL_FONT_FAMILY,
  TERMINAL_FONT_PROBE,
} from "@/features/terminal-session/model/terminal-font";

describe("terminal font loading", () => {
  it("loads the terminal font with representative TUI glyphs before rendering", async () => {
    const load = vi.fn().mockResolvedValue([]);

    await loadTerminalFont({ load } as unknown as FontFaceSet);

    expect(load).toHaveBeenCalledOnce();
    expect(load).toHaveBeenCalledWith(`13px ${TERMINAL_FONT_FAMILY}`, TERMINAL_FONT_PROBE);
    expect(TERMINAL_FONT_PROBE).toContain("─");
    expect(TERMINAL_FONT_PROBE).toContain("█");
    expect(TERMINAL_FONT_PROBE).toContain("❯");
    expect(TERMINAL_FONT_PROBE).toContain("한");
  });
});
