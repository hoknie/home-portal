import { QueryClient } from "@tanstack/react-query";

import { RequestError } from "./errors";

export function createQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        refetchOnWindowFocus: true,
        refetchIntervalInBackground: false,
        retry: (failures, error) => !(error instanceof RequestError && error.status < 500) && failures < 2,
      },
      mutations: { retry: false },
    },
  });
}
