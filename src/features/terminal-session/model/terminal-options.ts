import type { ITerminalOptions } from "@xterm/xterm";

import {
  TERMINAL_FALLBACK_FONT_FAMILY,
  TERMINAL_FONT_SIZE,
} from "@/features/terminal-session/model/terminal-font";

export function createTerminalOptions(): ITerminalOptions {
  return {
    allowProposedApi: true,
    allowTransparency: true,
    cursorBlink: true,
    cursorStyle: "bar",
    cursorWidth: 2,
    customGlyphs: true,
    disableStdin: false,
    fontFamily: TERMINAL_FALLBACK_FONT_FAMILY,
    fontSize: TERMINAL_FONT_SIZE,
    letterSpacing: 0,
    lineHeight: 1.2,
    macOptionClickForcesSelection: true,
    scrollback: 10_000,
    theme: {
      background: "#0b0e0c",
      foreground: "#d8ded9",
      cursor: "#b8ff4f",
      cursorAccent: "#0b0e0c",
      selectionBackground: "#b8ff4f30",
    },
  };
}
