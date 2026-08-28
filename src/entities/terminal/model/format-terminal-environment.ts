import type { TerminalEnvironment } from "@/entities/terminal/model/types";

export function getShellName(shellPath: string): string {
  const segments = shellPath.split(/[\\/]/).filter(Boolean);
  return segments[segments.length - 1] ?? shellPath;
}

export function formatRuntime(environment: TerminalEnvironment): string {
  return `${environment.platform} / ${environment.architecture}`;
}
