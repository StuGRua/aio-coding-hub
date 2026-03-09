import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { AppSettings } from "../../services/settings";
import { settingsGet, settingsSet } from "../../services/settings";
import { settingsCircuitBreakerNoticeSet } from "../../services/settingsCircuitBreakerNotice";
import { settingsCodexSessionIdCompletionSet } from "../../services/settingsCodexSessionIdCompletion";
import { settingsGatewayRectifierSet } from "../../services/settingsGatewayRectifier";
import { createQueryWrapper, createTestQueryClient } from "../../test/utils/reactQuery";
import { setTauriRuntime } from "../../test/utils/tauriRuntime";
import { settingsKeys } from "../keys";
import {
  useSettingsCircuitBreakerNoticeSetMutation,
  useSettingsCodexSessionIdCompletionSetMutation,
  useSettingsGatewayRectifierSetMutation,
  useSettingsQuery,
  useSettingsSetMutation,
} from "../settings";

vi.mock("../../services/settings", async () => {
  const actual =
    await vi.importActual<typeof import("../../services/settings")>("../../services/settings");
  return { ...actual, settingsGet: vi.fn(), settingsSet: vi.fn() };
});
vi.mock("../../services/settingsGatewayRectifier", async () => {
  const actual = await vi.importActual<typeof import("../../services/settingsGatewayRectifier")>(
    "../../services/settingsGatewayRectifier"
  );
  return { ...actual, settingsGatewayRectifierSet: vi.fn() };
});
vi.mock("../../services/settingsCircuitBreakerNotice", async () => {
  const actual = await vi.importActual<
    typeof import("../../services/settingsCircuitBreakerNotice")
  >("../../services/settingsCircuitBreakerNotice");
  return { ...actual, settingsCircuitBreakerNoticeSet: vi.fn() };
});
vi.mock("../../services/settingsCodexSessionIdCompletion", async () => {
  const actual = await vi.importActual<
    typeof import("../../services/settingsCodexSessionIdCompletion")
  >("../../services/settingsCodexSessionIdCompletion");
  return { ...actual, settingsCodexSessionIdCompletionSet: vi.fn() };
});

function makeSettings(overrides?: Partial<AppSettings>): AppSettings {
  return {
    schema_version: 1,
    preferred_port: 37123,
    gateway_listen_mode: "localhost",
    gateway_custom_listen_address: "",
    wsl_auto_config: false,
    wsl_target_cli: { claude: true, codex: true, gemini: true },
    wsl_host_address_mode: "auto",
    wsl_custom_host_address: "127.0.0.1",
    auto_start: false,
    tray_enabled: true,
    silent_startup: false,
    enable_cli_proxy_startup_recovery: true,
    log_retention_days: 7,
    provider_cooldown_seconds: 30,
    provider_base_url_ping_cache_ttl_seconds: 60,
    upstream_first_byte_timeout_seconds: 0,
    upstream_stream_idle_timeout_seconds: 0,
    upstream_request_timeout_non_streaming_seconds: 0,
    update_releases_url: "",
    failover_max_attempts_per_provider: 5,
    failover_max_providers_to_try: 5,
    circuit_breaker_failure_threshold: 5,
    circuit_breaker_open_duration_minutes: 30,
    enable_circuit_breaker_notice: false,
    verbose_provider_error: true,
    intercept_anthropic_warmup_requests: false,
    enable_thinking_signature_rectifier: true,
    enable_thinking_budget_rectifier: true,
    enable_codex_session_id_completion: true,
    enable_claude_metadata_user_id_injection: true,
    enable_cache_anomaly_monitor: false,
    enable_task_complete_notify: true,
    enable_response_fixer: true,
    response_fixer_fix_encoding: true,
    response_fixer_fix_sse_format: true,
    response_fixer_fix_truncated_json: true,
    response_fixer_max_json_depth: 200,
    response_fixer_max_fix_size: 1024,
    ...overrides,
  };
}

describe("query/settings", () => {
  it("does not call settingsGet without tauri runtime", async () => {
    const client = createTestQueryClient();
    const wrapper = createQueryWrapper(client);

    renderHook(() => useSettingsQuery(), { wrapper });
    await Promise.resolve();

    expect(settingsGet).not.toHaveBeenCalled();
  });

  it("calls settingsGet with tauri runtime", async () => {
    setTauriRuntime();
    vi.mocked(settingsGet).mockResolvedValue(makeSettings());

    const client = createTestQueryClient();
    const wrapper = createQueryWrapper(client);

    renderHook(() => useSettingsQuery(), { wrapper });

    await waitFor(() => {
      expect(settingsGet).toHaveBeenCalled();
    });
  });

  it("useSettingsQuery enters error state when settingsGet rejects", async () => {
    setTauriRuntime();
    vi.mocked(settingsGet).mockRejectedValue(new Error("settings query boom"));

    const client = createTestQueryClient();
    const wrapper = createQueryWrapper(client);

    const { result } = renderHook(() => useSettingsQuery(), { wrapper });
    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });
  });

  it("useSettingsSetMutation updates cache and invalidates on settle", async () => {
    setTauriRuntime();

    const updated = makeSettings({ preferred_port: 40000 });
    vi.mocked(settingsSet).mockResolvedValue(updated);

    const client = createTestQueryClient();
    client.setQueryData(settingsKeys.get(), makeSettings());
    const invalidateSpy = vi.spyOn(client, "invalidateQueries");
    const wrapper = createQueryWrapper(client);

    const { result } = renderHook(() => useSettingsSetMutation(), { wrapper });
    await act(async () => {
      await result.current.mutateAsync({
        preferredPort: 40000,
        autoStart: false,
        trayEnabled: true,
        logRetentionDays: 30,
        providerCooldownSeconds: 30,
        providerBaseUrlPingCacheTtlSeconds: 60,
        upstreamFirstByteTimeoutSeconds: 0,
        upstreamStreamIdleTimeoutSeconds: 0,
        upstreamRequestTimeoutNonStreamingSeconds: 0,
        enableCacheAnomalyMonitor: false,
        failoverMaxAttemptsPerProvider: 5,
        failoverMaxProvidersToTry: 5,
        circuitBreakerFailureThreshold: 5,
        circuitBreakerOpenDurationMinutes: 30,
      });
    });

    expect(client.getQueryData(settingsKeys.get())).toEqual(updated);
    expect(invalidateSpy).toHaveBeenCalledWith({ queryKey: settingsKeys.get() });
  });

  it("useSettingsGatewayRectifierSetMutation updates cache", async () => {
    setTauriRuntime();

    const updated = makeSettings({ enable_response_fixer: false });
    vi.mocked(settingsGatewayRectifierSet).mockResolvedValue(updated);

    const client = createTestQueryClient();
    client.setQueryData(settingsKeys.get(), makeSettings());
    const wrapper = createQueryWrapper(client);

    const { result } = renderHook(() => useSettingsGatewayRectifierSetMutation(), { wrapper });
    await act(async () => {
      await result.current.mutateAsync({ enable_response_fixer: false } as any);
    });

    expect(client.getQueryData(settingsKeys.get())).toEqual(updated);
  });

  it("useSettingsCircuitBreakerNoticeSetMutation updates cache", async () => {
    setTauriRuntime();

    const updated = makeSettings({ enable_circuit_breaker_notice: true });
    vi.mocked(settingsCircuitBreakerNoticeSet).mockResolvedValue(updated);

    const client = createTestQueryClient();
    client.setQueryData(settingsKeys.get(), makeSettings());
    const wrapper = createQueryWrapper(client);

    const { result } = renderHook(() => useSettingsCircuitBreakerNoticeSetMutation(), { wrapper });
    await act(async () => {
      await result.current.mutateAsync(true);
    });

    expect(client.getQueryData(settingsKeys.get())).toEqual(updated);
  });

  it("useSettingsCodexSessionIdCompletionSetMutation updates cache", async () => {
    setTauriRuntime();

    const updated = makeSettings({ enable_codex_session_id_completion: false });
    vi.mocked(settingsCodexSessionIdCompletionSet).mockResolvedValue(updated);

    const client = createTestQueryClient();
    client.setQueryData(settingsKeys.get(), makeSettings());
    const wrapper = createQueryWrapper(client);

    const { result } = renderHook(() => useSettingsCodexSessionIdCompletionSetMutation(), {
      wrapper,
    });
    await act(async () => {
      await result.current.mutateAsync(false);
    });

    expect(client.getQueryData(settingsKeys.get())).toEqual(updated);
  });
});
