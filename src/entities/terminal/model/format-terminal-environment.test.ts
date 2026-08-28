import { describe, expect, it } from "vitest";

import {
  formatRuntime,
  getShellName,
} from "@/entities/terminal/model/format-terminal-environment";

describe("terminal environment formatting", () => {
  it("extracts Unix and Windows shell names", () => {
    expect(getShellName("/bin/zsh")).toBe("zsh");
    expect(getShellName("C:\\Windows\\System32\\cmd.exe")).toBe("cmd.exe");
  });

  it("formats the runtime label", () => {
    expect(
      formatRuntime({
        defaultShell: "/bin/zsh",
        platform: "macos",
        architecture: "aarch64",
        workingDirectory: "/tmp",
      }),
    ).toBe("macos / aarch64");
  });
});
