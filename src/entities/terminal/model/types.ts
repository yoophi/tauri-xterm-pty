export type TerminalEnvironment = {
  defaultShell: string;
  platform: string;
  architecture: string;
  workingDirectory: string;
};

export type TerminalChannelEvent =
  | { type: "output"; data: number[] }
  | { type: "exited"; exitCode: number | null; signal: string | null }
  | { type: "error"; message: string };

export type TerminalSessionState =
  | { phase: "starting" }
  | { phase: "running"; sessionId: string }
  | { phase: "exited"; exitCode: number | null; signal: string | null }
  | { phase: "error"; message: string };

export type TerminalDimensions = {
  cols: number;
  rows: number;
};
