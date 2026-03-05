import { beforeEach, describe, expect, it, vi } from "vitest";

const { mockHasTauriRuntime, mockLogToConsole, mockListen } = vi.hoisted(() => ({
  mockHasTauriRuntime: vi.fn(() => true),
  mockLogToConsole: vi.fn(),
  mockListen: vi.fn(),
}));

vi.mock("../tauriInvoke", () => ({
  hasTauriRuntime: mockHasTauriRuntime,
}));

vi.mock("../consoleLog", () => ({
  logToConsole: mockLogToConsole,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: mockListen,
}));

async function importFreshGatewayEventBus() {
  vi.resetModules();
  return import("../gatewayEventBus");
}

type ListenCallback = (evt: { payload: unknown }) => void;

describe("services/gatewayEventBus", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockHasTauriRuntime.mockReturnValue(true);
  });

  it("returns noop subscription when tauri runtime is unavailable", async () => {
    mockHasTauriRuntime.mockReturnValue(false);
    const { subscribeGatewayEvent } = await importFreshGatewayEventBus();

    const handler = vi.fn();
    const { ready, unsubscribe } = subscribeGatewayEvent("gateway:status", handler);

    await expect(ready).resolves.toBeUndefined();
    expect(mockListen).not.toHaveBeenCalled();
    expect(() => unsubscribe()).not.toThrow();
  });

  it("reuses one listen registration for same event and cleans up on last unsubscribe", async () => {
    const mockUnlisten = vi.fn();
    let payloadHandler: ListenCallback | undefined;
    mockListen.mockImplementation(async (_event: string, cb: ListenCallback) => {
      payloadHandler = cb;
      return mockUnlisten;
    });

    const { subscribeGatewayEvent } = await importFreshGatewayEventBus();
    const handlerA = vi.fn();
    const handlerB = vi.fn();

    const subA = subscribeGatewayEvent("gateway:status", handlerA);
    const subB = subscribeGatewayEvent("gateway:status", handlerB);
    await Promise.all([subA.ready, subB.ready]);

    expect(mockListen).toHaveBeenCalledTimes(1);
    expect(payloadHandler).toBeTypeOf("function");
    (payloadHandler as ListenCallback)({ payload: { ok: true } });
    expect(handlerA).toHaveBeenCalledWith({ ok: true });
    expect(handlerB).toHaveBeenCalledWith({ ok: true });

    subA.unsubscribe();
    expect(mockUnlisten).not.toHaveBeenCalled();
    subB.unsubscribe();
    expect(mockUnlisten).toHaveBeenCalledTimes(1);
  });

  it("unsubscribes immediately after delayed listen init completes", async () => {
    let resolveListener: ((unlisten: () => void) => void) | undefined;
    const listenerPromise = new Promise<() => void>((resolve) => {
      resolveListener = resolve;
    });
    const mockUnlisten = vi.fn();
    mockListen.mockReturnValue(listenerPromise);

    const { subscribeGatewayEvent } = await importFreshGatewayEventBus();
    const { unsubscribe, ready } = subscribeGatewayEvent("gateway:log", vi.fn());

    unsubscribe();
    expect(resolveListener).toBeTypeOf("function");
    (resolveListener as (unlisten: () => void) => void)(mockUnlisten);
    await ready;

    expect(mockUnlisten).toHaveBeenCalledTimes(1);
  });

  it("logs error when listen initialization fails", async () => {
    mockListen.mockRejectedValueOnce(new Error("listen boom"));
    const { subscribeGatewayEvent } = await importFreshGatewayEventBus();

    const { ready } = subscribeGatewayEvent("gateway:circuit", vi.fn());
    await expect(ready).resolves.toBeUndefined();

    expect(mockLogToConsole).toHaveBeenCalledWith(
      "error",
      "网关事件监听初始化失败",
      expect.objectContaining({
        event: "gateway:circuit",
        error: expect.stringContaining("listen boom"),
      }),
      "gateway:event_bus"
    );
  });
});
