import { render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { QueryClientProvider } from "@tanstack/react-query";
import { createTestQueryClient } from "../test/utils/reactQuery";
import App from "../App";

const { mockLogToConsole } = vi.hoisted(() => ({
  mockLogToConsole: vi.fn(),
}));

vi.mock("../services/consoleLog", async () => {
  const actual =
    await vi.importActual<typeof import("../services/consoleLog")>("../services/consoleLog");
  return {
    ...actual,
    logToConsole: mockLogToConsole,
  };
});

vi.mock("../services/gatewayEvents", () => ({
  listenGatewayEvents: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("../services/noticeEvents", () => ({
  listenNoticeEvents: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("../services/appHeartbeat", () => ({
  listenAppHeartbeat: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("../services/taskCompleteNotifyEvents", () => ({
  listenTaskCompleteNotifyEvents: vi.fn().mockResolvedValue(() => {}),
  setTaskCompleteNotifyEnabled: vi.fn(),
}));

vi.mock("../services/cacheAnomalyMonitor", () => ({
  setCacheAnomalyMonitorEnabled: vi.fn(),
}));

vi.mock("../services/startup", () => ({
  startupSyncDefaultPromptsFromFilesOncePerSession: vi.fn().mockResolvedValue(undefined),
  startupSyncModelPricesOnce: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("../services/settings", async () => {
  const actual =
    await vi.importActual<typeof import("../services/settings")>("../services/settings");
  return {
    ...actual,
    settingsGet: vi.fn().mockResolvedValue(null),
  };
});

import { listenGatewayEvents } from "../services/gatewayEvents";
import { listenNoticeEvents } from "../services/noticeEvents";
import { settingsGet } from "../services/settings";
import { listenAppHeartbeat } from "../services/appHeartbeat";
import {
  listenTaskCompleteNotifyEvents,
  setTaskCompleteNotifyEnabled,
} from "../services/taskCompleteNotifyEvents";
import { setCacheAnomalyMonitorEnabled } from "../services/cacheAnomalyMonitor";
import {
  startupSyncDefaultPromptsFromFilesOncePerSession,
  startupSyncModelPricesOnce,
} from "../services/startup";

const DEFAULT_HASH = "#/";

function renderApp() {
  const client = createTestQueryClient();
  return render(
    <QueryClientProvider client={client}>
      <App />
    </QueryClientProvider>
  );
}

async function renderRouteAndFindHeading(hash: string, headingName: string) {
  window.location.hash = hash;
  renderApp();
  return screen.findByRole("heading", { level: 1, name: headingName });
}

describe("App (smoke)", () => {
  beforeEach(() => {
    mockLogToConsole.mockReset();
    vi.mocked(listenAppHeartbeat).mockResolvedValue(() => {});
    vi.mocked(listenGatewayEvents).mockResolvedValue(() => {});
    vi.mocked(listenNoticeEvents).mockResolvedValue(() => {});
    vi.mocked(listenTaskCompleteNotifyEvents).mockResolvedValue(() => {});
    vi.mocked(startupSyncModelPricesOnce).mockResolvedValue(undefined);
    vi.mocked(startupSyncDefaultPromptsFromFilesOncePerSession).mockResolvedValue(undefined);
    vi.mocked(settingsGet).mockResolvedValue(null);
    vi.mocked(setCacheAnomalyMonitorEnabled).mockReset();
    vi.mocked(setTaskCompleteNotifyEnabled).mockReset();
  });

  afterEach(() => {
    window.location.hash = DEFAULT_HASH;
  });

  it("renders home route by default", async () => {
    expect(await renderRouteAndFindHeading("#/", "首页")).toBeInTheDocument();
  });

  it("renders settings route via hash", async () => {
    expect(await renderRouteAndFindHeading("#/settings", "设置")).toBeInTheDocument();
  });

  it("logs warning when event listeners initialization fails", async () => {
    vi.mocked(listenGatewayEvents).mockRejectedValueOnce(new Error("gateway init failed"));
    vi.mocked(listenNoticeEvents).mockRejectedValueOnce(new Error("notice init failed"));

    window.location.hash = "#/settings";
    renderApp();

    expect(await screen.findByRole("heading", { level: 1, name: "设置" })).toBeInTheDocument();

    await vi.waitFor(() => {
      expect(mockLogToConsole).toHaveBeenCalledWith(
        "warn",
        "网关事件监听初始化失败",
        expect.objectContaining({
          stage: "listenGatewayEvents",
          error: expect.stringContaining("gateway init failed"),
        })
      );
    });

    expect(mockLogToConsole).toHaveBeenCalledWith(
      "warn",
      "通知事件监听初始化失败",
      expect.objectContaining({
        stage: "listenNoticeEvents",
        error: expect.stringContaining("notice init failed"),
      })
    );
  });

  it("syncs startup switches from settings when settings are available", async () => {
    vi.mocked(settingsGet).mockResolvedValue({
      enable_cache_anomaly_monitor: true,
      enable_task_complete_notify: false,
    } as any);

    renderApp();
    expect(await screen.findByRole("heading", { level: 1, name: "首页" })).toBeInTheDocument();

    await vi.waitFor(() => {
      expect(setCacheAnomalyMonitorEnabled).toHaveBeenCalledWith(true);
      expect(setTaskCompleteNotifyEnabled).toHaveBeenCalledWith(false);
    });
  });

  it("logs warning when startup settings sync fails", async () => {
    vi.mocked(settingsGet).mockRejectedValueOnce(new Error("settings init failed"));

    renderApp();
    expect(await screen.findByRole("heading", { level: 1, name: "首页" })).toBeInTheDocument();

    await vi.waitFor(() => {
      expect(mockLogToConsole).toHaveBeenCalledWith(
        "warn",
        "启动缓存异常监测开关同步失败",
        expect.objectContaining({
          stage: "startupSyncCacheAnomalyMonitorSwitch",
          error: expect.stringContaining("settings init failed"),
        })
      );
    });
  });
});
