import { useEffect, useRef } from "react";
import { FitAddon } from "@xterm/addon-fit";
import { Unicode11Addon } from "@xterm/addon-unicode11";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

import {
  resizeTerminal,
  startTerminal,
  stopTerminal,
  writeTerminal,
} from "@/entities/terminal/api/terminal-api";
import type { TerminalChannelEvent } from "@/entities/terminal/model/types";
import {
  loadTerminalFont,
  TERMINAL_FONT_FAMILY,
} from "@/features/terminal-session/model/terminal-font";
import { createTerminalOptions } from "@/features/terminal-session/model/terminal-options";
import { enableWebglRenderer } from "@/features/terminal-session/model/webgl-renderer";

const textEncoder = new TextEncoder();

export function TerminalSurface() {
  const hostRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) {
      return;
    }

    const sessionId = crypto.randomUUID();
    let isActive = true;
    let hasStarted = false;
    let hasFinished = false;
    let resizeTimer: ReturnType<typeof setTimeout> | undefined;
    let inputQueue = Promise.resolve();

    const terminal = new Terminal(createTerminalOptions());
    const fitAddon = new FitAddon();
    const unicode11Addon = new Unicode11Addon();
    terminal.loadAddon(unicode11Addon);
    terminal.unicode.activeVersion = "11";
    terminal.loadAddon(fitAddon);
    terminal.open(host);
    enableWebglRenderer(terminal);

    const reportError = (reason: unknown) => {
      const message = reason instanceof Error ? reason.message : String(reason);
      if (isActive) {
        hasFinished = true;
        terminal.writeln(`\r\n\x1b[38;2;255;117;107m[terminal error]\x1b[0m ${message}`);
        terminal.options.disableStdin = true;
      }
    };

    const handleEvent = (event: TerminalChannelEvent) => {
      if (!isActive) {
        return;
      }

      switch (event.type) {
        case "output":
          terminal.write(Uint8Array.from(event.data));
          break;
        case "exited": {
          hasFinished = true;
          terminal.options.disableStdin = true;
          const reason = event.signal
            ? `signal ${event.signal}`
            : `code ${event.exitCode ?? "unknown"}`;
          terminal.writeln(`\r\n\x1b[38;2;104;112;106m[process exited: ${reason}]\x1b[0m`);
          break;
        }
        case "error":
          reportError(event.message);
          break;
      }
    };

    const fit = () => {
      try {
        fitAddon.fit();
      } catch {
        // The host may be between layout states while the window is resizing.
      }
    };

    void loadTerminalFont()
      .then(() => {
        if (!isActive) {
          return;
        }
        terminal.options.fontFamily = TERMINAL_FONT_FAMILY;
        fit();
        terminal.refresh(0, terminal.rows - 1);
      })
      .catch(() => {
        // The terminal remains usable with the system monospace fallback.
      });

    const dataDisposable = terminal.onData((data) => {
      if (!hasStarted || !isActive) {
        return;
      }
      const bytes = textEncoder.encode(data);
      inputQueue = inputQueue
        .then(() => (isActive ? writeTerminal(sessionId, bytes) : undefined))
        .catch(reportError);
    });

    const resizeDisposable = terminal.onResize((dimensions) => {
      if (!hasStarted || !isActive) {
        return;
      }
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(() => {
        void resizeTerminal(sessionId, dimensions).catch(reportError);
      }, 40);
    });

    const animationFrame = requestAnimationFrame(() => {
      fit();
      const dimensions = { cols: terminal.cols, rows: terminal.rows };

      void startTerminal({ sessionId, ...dimensions }, handleEvent)
        .then(() => {
          hasStarted = true;
          if (!isActive) {
            return stopTerminal(sessionId).catch(() => undefined);
          }
          if (!hasFinished) {
            terminal.focus();
          }
        })
        .catch(reportError);
    });

    const observer = new ResizeObserver(fit);
    observer.observe(host);

    return () => {
      isActive = false;
      cancelAnimationFrame(animationFrame);
      clearTimeout(resizeTimer);
      observer.disconnect();
      dataDisposable.dispose();
      resizeDisposable.dispose();
      terminal.dispose();
      void stopTerminal(sessionId).catch(() => undefined);
    };
  }, []);

  return (
    <div
      ref={hostRef}
      role="region"
      aria-label="Interactive local terminal"
      className="size-full overflow-hidden"
    />
  );
}
