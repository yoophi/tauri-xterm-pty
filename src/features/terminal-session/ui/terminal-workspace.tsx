import { TerminalSurface } from "@/features/terminal-session/ui/terminal-surface";

export function TerminalWorkspace() {
  return (
    <main className="h-dvh w-screen overflow-hidden bg-[#0b0e0c]" aria-label="Terminal">
      <TerminalSurface />
    </main>
  );
}
