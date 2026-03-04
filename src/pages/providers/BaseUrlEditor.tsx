// Usage: Used by ProviderEditorDialog to edit, ping and stream-check multiple Base URLs.

import { useEffect, useRef, type Dispatch, type SetStateAction } from "react";
import { toast } from "sonner";
import {
  baseUrlPingMs,
  providerStreamCheck,
  type ProviderStreamCheckResult,
} from "../../services/providers";
import { Button } from "../../ui/Button";
import { Input } from "../../ui/Input";
import { cn } from "../../utils/cn";
import type { BaseUrlRow } from "./types";

export type BaseUrlEditorProps = {
  rows: BaseUrlRow[];
  setRows: Dispatch<SetStateAction<BaseUrlRow[]>>;
  pingingAll: boolean;
  setPingingAll: Dispatch<SetStateAction<boolean>>;
  newRow: (url?: string) => BaseUrlRow;
  disabled?: boolean;
  placeholder?: string;
  cliKey?: string;
  apiKey?: string;
  providerId?: number;
  testModel?: string;
};

async function pingBaseUrlRow(
  rowId: string,
  url: string,
  setRows: Dispatch<SetStateAction<BaseUrlRow[]>>
) {
  const baseUrl = url.trim();
  if (!baseUrl) {
    toast("Base URL 不能为空");
    return;
  }

  setRows((prev) =>
    prev.map((row) => (row.id === rowId ? { ...row, ping: { status: "pinging" } } : row))
  );

  try {
    const ms = await baseUrlPingMs(baseUrl);
    if (ms == null) {
      toast("仅在 Tauri Desktop 环境可用");
      setRows((prev) =>
        prev.map((row) =>
          row.id === rowId && row.url.trim() === baseUrl
            ? { ...row, ping: { status: "idle" } }
            : row
        )
      );
      return;
    }

    setRows((prev) =>
      prev.map((row) =>
        row.id === rowId && row.url.trim() === baseUrl
          ? { ...row, ping: { status: "ok", ms } }
          : row
      )
    );
  } catch (err) {
    const message = String(err);
    setRows((prev) =>
      prev.map((row) =>
        row.id === rowId && row.url.trim() === baseUrl
          ? { ...row, ping: { status: "error", message } }
          : row
      )
    );
  }
}

async function pingAllBaseUrlRows(
  rowsSnapshot: BaseUrlRow[],
  setRows: Dispatch<SetStateAction<BaseUrlRow[]>>,
  setPingingAll: Dispatch<SetStateAction<boolean>>
) {
  if (rowsSnapshot.length === 0) return;
  setPingingAll(true);
  try {
    for (const row of rowsSnapshot) {
      await pingBaseUrlRow(row.id, row.url, setRows);
    }
  } finally {
    setPingingAll(false);
  }
}

function applyStreamCheckResult(
  rowId: string,
  baseUrl: string,
  result: ProviderStreamCheckResult,
  setRows: Dispatch<SetStateAction<BaseUrlRow[]>>
) {
  setRows((prev) =>
    prev.map((row) => {
      if (row.id !== rowId || row.url.trim() !== baseUrl) return row;
      if (result.ok) {
        const status = result.grade === "degraded" ? "degraded" : "operational";
        return { ...row, streamCheck: { status, ms: result.duration_ms } };
      }
      return {
        ...row,
        streamCheck: {
          status: "failed" as const,
          message: result.message ?? "unknown error",
          failureKind: result.failure_kind ?? "unknown",
        },
      };
    })
  );
}

export function BaseUrlEditor({
  rows,
  setRows,
  pingingAll,
  setPingingAll,
  newRow,
  disabled,
  placeholder,
  cliKey,
  apiKey,
  providerId,
  testModel,
}: BaseUrlEditorProps) {
  const abortControllersRef = useRef<Map<string, AbortController>>(new Map());

  // Cleanup all in-flight requests on unmount
  useEffect(() => {
    const controllers = abortControllersRef.current;
    return () => {
      for (const controller of controllers.values()) {
        controller.abort();
      }
    };
  }, []);

  async function handleStreamCheck(rowId: string, url: string) {
    const baseUrl = url.trim();
    if (!baseUrl || !cliKey) return;

    // Abort previous request for this row
    const prevController = abortControllersRef.current.get(rowId);
    if (prevController) prevController.abort();

    const controller = new AbortController();
    abortControllersRef.current.set(rowId, controller);

    setRows((prev) =>
      prev.map((row) => (row.id === rowId ? { ...row, streamCheck: { status: "checking" } } : row))
    );

    try {
      const result = await providerStreamCheck({
        cli_key: cliKey,
        base_url: baseUrl,
        api_key: apiKey || undefined,
        provider_id: providerId,
        model: testModel || undefined,
      });

      if (controller.signal.aborted) return;

      if (result == null) {
        toast("仅在 Tauri Desktop 环境可用");
        setRows((prev) =>
          prev.map((row) => (row.id === rowId ? { ...row, streamCheck: { status: "idle" } } : row))
        );
        return;
      }

      applyStreamCheckResult(rowId, baseUrl, result, setRows);
    } catch (err) {
      if (controller.signal.aborted) return;
      setRows((prev) =>
        prev.map((row) =>
          row.id === rowId && row.url.trim() === baseUrl
            ? {
                ...row,
                streamCheck: {
                  status: "failed" as const,
                  message: String(err),
                  failureKind: "unknown",
                },
              }
            : row
        )
      );
    } finally {
      abortControllersRef.current.delete(rowId);
    }
  }

  return (
    <div className="space-y-2">
      {rows.map((row, index) => {
        const canMoveUp = index > 0;
        const canMoveDown = index < rows.length - 1;
        const removeDisabled = rows.length <= 1;
        const pinging = row.ping.status === "pinging";
        const checking = row.streamCheck.status === "checking";

        // Stream check button disabled: URL empty OR (no apiKey AND no providerId)
        const streamCheckDisabled =
          !row.url.trim() || (!apiKey?.trim() && providerId == null) || checking || disabled;

        const pingBadge =
          row.ping.status === "pinging" ? (
            <span className="text-xs text-slate-400">…</span>
          ) : row.ping.status === "ok" ? (
            <span className="font-mono text-xs text-emerald-600">{row.ping.ms}ms</span>
          ) : row.ping.status === "error" ? (
            <span className="text-xs text-rose-500" title={row.ping.message}>
              失败
            </span>
          ) : null;

        const streamBadge =
          row.streamCheck.status === "checking" ? (
            <span className="text-xs text-slate-400">…</span>
          ) : row.streamCheck.status === "operational" ? (
            <span className="font-mono text-xs text-emerald-600">{row.streamCheck.ms}ms</span>
          ) : row.streamCheck.status === "degraded" ? (
            <span className="font-mono text-xs text-amber-500">{row.streamCheck.ms}ms</span>
          ) : row.streamCheck.status === "failed" ? (
            <span className="text-xs text-rose-500" title={row.streamCheck.message}>
              失败
            </span>
          ) : null;

        const hasBadge = pingBadge || streamBadge;

        return (
          <div key={row.id} className="flex items-center gap-2">
            <div className="relative flex-1">
              <Input
                value={row.url}
                onChange={(e) => {
                  const nextValue = e.currentTarget.value;
                  setRows((prev) =>
                    prev.map((r) =>
                      r.id === row.id
                        ? {
                            ...r,
                            url: nextValue,
                            ping: { status: "idle" },
                            streamCheck: { status: "idle" },
                          }
                        : r
                    )
                  );
                }}
                placeholder={placeholder ?? "https://api.openai.com"}
                className={cn("w-full font-mono text-sm h-8 py-1", hasBadge ? "pr-24" : null)}
              />
              {hasBadge ? (
                <span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-2">
                  {pingBadge}
                  {streamBadge}
                </span>
              ) : null}
            </div>

            <div className="flex items-center">
              <Button
                onClick={() =>
                  setRows((prev) => {
                    if (!canMoveUp) return prev;
                    const next = prev.slice();
                    const a = next[index - 1];
                    next[index - 1] = next[index];
                    next[index] = a;
                    return next;
                  })
                }
                variant="secondary"
                size="sm"
                disabled={!canMoveUp || pingingAll || disabled}
                className="rounded-r-none border-r-0 h-8"
                title="上移"
              >
                ↑
              </Button>
              <Button
                onClick={() =>
                  setRows((prev) => {
                    if (!canMoveDown) return prev;
                    const next = prev.slice();
                    const a = next[index + 1];
                    next[index + 1] = next[index];
                    next[index] = a;
                    return next;
                  })
                }
                variant="secondary"
                size="sm"
                disabled={!canMoveDown || pingingAll || disabled}
                className="rounded-l-none h-8"
                title="下移"
              >
                ↓
              </Button>
            </div>

            <Button
              onClick={() => void pingBaseUrlRow(row.id, row.url, setRows)}
              variant="secondary"
              size="sm"
              disabled={pinging || pingingAll || disabled}
              className="h-8"
            >
              Ping
            </Button>

            <Button
              onClick={() => void handleStreamCheck(row.id, row.url)}
              variant="secondary"
              size="sm"
              disabled={streamCheckDisabled}
              className="h-8"
            >
              {checking ? "测试中…" : "测试"}
            </Button>

            <Button
              onClick={() =>
                setRows((prev) => (prev.length <= 1 ? prev : prev.filter((r) => r.id !== row.id)))
              }
              variant="secondary"
              size="sm"
              disabled={removeDisabled || pingingAll || disabled}
              className="hover:!bg-rose-50 hover:!text-rose-600 h-8"
            >
              ×
            </Button>
          </div>
        );
      })}

      <div className="flex items-center gap-2 pt-1">
        <Button
          onClick={() => setRows((prev) => [...prev, newRow()])}
          variant="secondary"
          size="sm"
          disabled={pingingAll || disabled}
          className="h-8"
        >
          + 添加
        </Button>
        <Button
          onClick={() => void pingAllBaseUrlRows(rows, setRows, setPingingAll)}
          variant="secondary"
          size="sm"
          disabled={pingingAll || rows.length === 0 || disabled}
          className="h-8"
        >
          {pingingAll ? "检测中…" : "全部 Ping"}
        </Button>
      </div>
    </div>
  );
}
