import { Channel, invoke } from "@tauri-apps/api/core";

import type {
  TerminalChannelEvent,
  TerminalDimensions,
  TerminalEnvironment,
} from "@/entities/terminal/model/types";

export function getTerminalEnvironment(): Promise<TerminalEnvironment> {
  return invoke<TerminalEnvironment>("get_terminal_environment");
}

type TerminalSessionInput = TerminalDimensions & {
  sessionId: string;
};

type StartTerminalResponse = {
  sessionId: string;
};

const MAX_INPUT_CHUNK_BYTES = 64 * 1024;

export function startTerminal(
  input: TerminalSessionInput,
  onEvent: (event: TerminalChannelEvent) => void,
): Promise<StartTerminalResponse> {
  const channel = new Channel<TerminalChannelEvent>(onEvent);
  return invoke<StartTerminalResponse>("start_terminal", {
    input,
    onEvent: channel,
  });
}

export async function writeTerminal(sessionId: string, data: Uint8Array): Promise<void> {
  for (let offset = 0; offset < data.length; offset += MAX_INPUT_CHUNK_BYTES) {
    const chunk = data.subarray(offset, offset + MAX_INPUT_CHUNK_BYTES);
    await invoke("write_terminal", {
      input: {
        sessionId,
        data: Array.from(chunk),
      },
    });
  }
}

export function resizeTerminal(
  sessionId: string,
  dimensions: TerminalDimensions,
): Promise<void> {
  return invoke("resize_terminal", {
    input: { sessionId, ...dimensions },
  });
}

export function stopTerminal(sessionId: string): Promise<void> {
  return invoke("stop_terminal", {
    input: { sessionId },
  });
}
