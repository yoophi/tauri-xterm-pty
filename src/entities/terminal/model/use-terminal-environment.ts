import { useCallback, useEffect, useState } from "react";

import { getTerminalEnvironment } from "@/entities/terminal/api/terminal-api";
import type { TerminalEnvironment } from "@/entities/terminal/model/types";

type TerminalEnvironmentState = {
  environment: TerminalEnvironment | null;
  error: string | null;
  isLoading: boolean;
  reload: () => void;
};

export function useTerminalEnvironment(): TerminalEnvironmentState {
  const [environment, setEnvironment] = useState<TerminalEnvironment | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [requestId, setRequestId] = useState(0);
  const [isLoading, setIsLoading] = useState(true);

  const reload = useCallback(() => {
    setRequestId((current) => current + 1);
  }, []);

  useEffect(() => {
    let isActive = true;
    setIsLoading(true);
    setError(null);

    void getTerminalEnvironment()
      .then((nextEnvironment) => {
        if (isActive) {
          setEnvironment(nextEnvironment);
        }
      })
      .catch((reason: unknown) => {
        if (isActive) {
          setError(reason instanceof Error ? reason.message : String(reason));
        }
      })
      .finally(() => {
        if (isActive) {
          setIsLoading(false);
        }
      });

    return () => {
      isActive = false;
    };
  }, [requestId]);

  return { environment, error, isLoading, reload };
}
