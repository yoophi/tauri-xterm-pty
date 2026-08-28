import { describe, expect, it } from "vitest";

import { getTerminalStatusView } from "@/features/terminal-session/model/session-status";

describe("terminal session status", () => {
  it("describes running and exited sessions", () => {
    expect(
      getTerminalStatusView({ phase: "running", sessionId: "session" }),
    ).toEqual({ label: "Interactive", tone: "online" });
    expect(
      getTerminalStatusView({ phase: "exited", exitCode: 130, signal: null }),
    ).toEqual({ label: "Exited · 130", tone: "offline" });
  });
});
