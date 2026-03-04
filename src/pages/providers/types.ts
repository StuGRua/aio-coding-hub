// Usage: Shared types for `src/pages/providers/*` modules.

export type ProviderBaseUrlMode = "order" | "ping";

export type BaseUrlPingState =
  | { status: "idle" }
  | { status: "pinging" }
  | { status: "ok"; ms: number }
  | { status: "error"; message: string };

export type StreamCheckState =
  | { status: "idle" }
  | { status: "checking" }
  | { status: "operational"; ms: number }
  | { status: "degraded"; ms: number }
  | { status: "failed"; message: string; failureKind: string };

export type BaseUrlRow = {
  id: string;
  url: string;
  ping: BaseUrlPingState;
  streamCheck: StreamCheckState;
};
