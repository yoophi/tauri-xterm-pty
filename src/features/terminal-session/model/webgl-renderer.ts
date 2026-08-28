import { WebglAddon } from "@xterm/addon-webgl";
import type { Terminal } from "@xterm/xterm";

type WebglAddonFactory = () => WebglAddon;
type TerminalAddonHost = Pick<Terminal, "loadAddon">;

const createWebglAddon: WebglAddonFactory = () => new WebglAddon();

export function enableWebglRenderer(
  terminal: TerminalAddonHost,
  createAddon: WebglAddonFactory = createWebglAddon,
): boolean {
  let webglAddon: WebglAddon | undefined;

  try {
    webglAddon = createAddon();
    webglAddon.onContextLoss(() => {
      console.warn("[terminal] WebGL context lost; falling back to the DOM renderer.");
      webglAddon?.dispose();
    });
    terminal.loadAddon(webglAddon);
    return true;
  } catch (reason) {
    webglAddon?.dispose();
    console.warn(
      "[terminal] WebGL renderer unavailable; using the DOM renderer.",
      reason,
    );
    return false;
  }
}
