export const TERMINAL_FONT_SIZE = 13;
export const TERMINAL_FALLBACK_FONT_FAMILY = "ui-monospace, monospace";
export const TERMINAL_FONT_FAMILY =
  '"IBM Plex Mono Complete", "BlexMono Nerd Font Mono", "Noto Sans Symbols 2", "Apple Symbols", ui-monospace, monospace';

// Force the browser to resolve the font faces used by typical agentic TUI output
// before xterm measures and caches its character cell dimensions.
export const TERMINAL_FONT_PROBE = "M─█▝▛⏵✳▐▜▀◉❯←한🙂";

export async function loadTerminalFont(fonts: FontFaceSet = document.fonts): Promise<void> {
  await fonts.load(
    `${TERMINAL_FONT_SIZE}px ${TERMINAL_FONT_FAMILY}`,
    TERMINAL_FONT_PROBE,
  );
}
