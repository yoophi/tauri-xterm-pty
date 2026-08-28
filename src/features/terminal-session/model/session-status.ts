import type { TerminalSessionState } from "@/entities/terminal/model/types";

export type TerminalStatusView = {
  label: string;
  tone: "pending" | "online" | "offline" | "error";
};

export function getTerminalStatusView(state: TerminalSessionState): TerminalStatusView {
  switch (state.phase) {
    case "starting":
      return { label: "Starting", tone: "pending" };
    case "running":
      return { label: "Interactive", tone: "online" };
    case "exited":
      return {
        label: state.signal
          ? `Exited · ${state.signal}`
          : `Exited · ${state.exitCode ?? "?"}`,
        tone: "offline",
      };
    case "error":
      return { label: "Session fault", tone: "error" };
  }
}
