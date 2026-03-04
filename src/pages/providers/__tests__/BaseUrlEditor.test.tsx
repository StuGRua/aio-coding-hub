import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useState } from "react";
import { toast } from "sonner";
import { BaseUrlEditor } from "../BaseUrlEditor";
import type { BaseUrlRow } from "../types";
import { baseUrlPingMs, providerStreamCheck } from "../../../services/providers";

vi.mock("sonner", () => ({ toast: vi.fn() }));

vi.mock("../../../services/providers", async () => {
  const actual = await vi.importActual<typeof import("../../../services/providers")>(
    "../../../services/providers"
  );
  return { ...actual, baseUrlPingMs: vi.fn(), providerStreamCheck: vi.fn() };
});

function TestWrapper({
  initial,
  cliKey,
  apiKey,
  providerId,
  testModel,
}: {
  initial: BaseUrlRow[];
  cliKey?: string;
  apiKey?: string;
  providerId?: number;
  testModel?: string;
}) {
  const [rows, setRows] = useState<BaseUrlRow[]>(initial);
  const [pingingAll, setPingingAll] = useState(false);
  const newRow = (url = ""): BaseUrlRow => ({
    id: String(rows.length + 1),
    url,
    ping: { status: "idle" },
    streamCheck: { status: "idle" },
  });

  return (
    <BaseUrlEditor
      rows={rows}
      setRows={setRows}
      pingingAll={pingingAll}
      setPingingAll={setPingingAll}
      newRow={newRow}
      cliKey={cliKey}
      apiKey={apiKey}
      providerId={providerId}
      testModel={testModel}
    />
  );
}

describe("pages/providers/BaseUrlEditor", () => {
  it("pings base urls and handles empty/tauri-only/error cases", async () => {
    vi.mocked(baseUrlPingMs).mockResolvedValueOnce(123);
    vi.mocked(baseUrlPingMs).mockResolvedValueOnce(null);
    vi.mocked(baseUrlPingMs).mockRejectedValueOnce(new Error("boom"));

    render(
      <TestWrapper
        initial={[{ id: "1", url: "", ping: { status: "idle" }, streamCheck: { status: "idle" } }]}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Ping" }));
    expect(vi.mocked(toast)).toHaveBeenCalledWith("Base URL 不能为空");

    fireEvent.change(screen.getByPlaceholderText("https://api.openai.com"), {
      target: { value: "https://example.com" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Ping" }));
    await waitFor(() => expect(screen.getByText("123ms")).toBeInTheDocument());

    fireEvent.click(screen.getByRole("button", { name: "Ping" }));
    await waitFor(() =>
      expect(vi.mocked(toast)).toHaveBeenCalledWith("仅在 Tauri Desktop 环境可用")
    );

    fireEvent.click(screen.getByRole("button", { name: "Ping" }));
    await waitFor(() => expect(screen.getByText("失败")).toBeInTheDocument());
  });

  it("supports adding and pinging all rows", async () => {
    vi.mocked(baseUrlPingMs).mockResolvedValue(10);

    render(
      <TestWrapper
        initial={[
          { id: "1", url: "https://a", ping: { status: "idle" }, streamCheck: { status: "idle" } },
        ]}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "+ 添加" }));
    const inputs = screen.getAllByPlaceholderText("https://api.openai.com");
    fireEvent.change(inputs[1]!, { target: { value: "https://b" } });

    fireEvent.click(screen.getByRole("button", { name: "全部 Ping" }));
    await waitFor(() => expect(screen.getAllByText(/ms$/).length).toBeGreaterThanOrEqual(2));
  });

  it("runs stream check and shows operational badge", async () => {
    vi.mocked(providerStreamCheck).mockResolvedValueOnce({
      ok: true,
      grade: "operational",
      duration_ms: 456,
      target_url: "https://example.com/v1/chat/completions",
      used_model: "gpt-4.1-mini",
      attempts: 1,
    });

    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://example.com",
            ping: { status: "idle" },
            streamCheck: { status: "idle" },
          },
        ]}
        cliKey="claude"
        apiKey="sk-test"
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "测试" }));
    await waitFor(() => expect(screen.getByText("456ms")).toBeInTheDocument());
  });

  it("runs stream check and shows degraded badge", async () => {
    vi.mocked(providerStreamCheck).mockResolvedValueOnce({
      ok: true,
      grade: "degraded",
      duration_ms: 3200,
      target_url: "https://example.com/v1/chat/completions",
      used_model: "gpt-4.1-mini",
      attempts: 1,
    });

    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://example.com",
            ping: { status: "idle" },
            streamCheck: { status: "idle" },
          },
        ]}
        cliKey="claude"
        apiKey="sk-test"
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "测试" }));
    await waitFor(() => expect(screen.getByText("3200ms")).toBeInTheDocument());
  });

  it("runs stream check and shows failed badge on error result", async () => {
    vi.mocked(providerStreamCheck).mockResolvedValueOnce({
      ok: false,
      grade: "failed",
      duration_ms: 0,
      target_url: "https://example.com/v1/chat/completions",
      used_model: "gpt-4.1-mini",
      failure_kind: "auth",
      message: "401 Unauthorized",
      attempts: 1,
    });

    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://example.com",
            ping: { status: "idle" },
            streamCheck: { status: "idle" },
          },
        ]}
        cliKey="claude"
        apiKey="sk-test"
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "测试" }));
    await waitFor(() => {
      const failBadges = screen.getAllByText("失败");
      expect(failBadges.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("shows toast when stream check returns null (non-Tauri)", async () => {
    vi.mocked(providerStreamCheck).mockResolvedValueOnce(null as never);

    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://example.com",
            ping: { status: "idle" },
            streamCheck: { status: "idle" },
          },
        ]}
        cliKey="claude"
        apiKey="sk-test"
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "测试" }));
    await waitFor(() =>
      expect(vi.mocked(toast)).toHaveBeenCalledWith("仅在 Tauri Desktop 环境可用")
    );
  });

  it("handles stream check rejection gracefully", async () => {
    vi.mocked(providerStreamCheck).mockRejectedValueOnce(new Error("network"));

    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://example.com",
            ping: { status: "idle" },
            streamCheck: { status: "idle" },
          },
        ]}
        cliKey="claude"
        apiKey="sk-test"
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "测试" }));
    await waitFor(() => {
      const failBadges = screen.getAllByText("失败");
      expect(failBadges.length).toBeGreaterThanOrEqual(1);
    });
  });

  it("disables stream check button when no apiKey and no providerId", () => {
    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://example.com",
            ping: { status: "idle" },
            streamCheck: { status: "idle" },
          },
        ]}
        cliKey="claude"
      />
    );

    const testBtn = screen.getByRole("button", { name: "测试" });
    expect(testBtn).toBeDisabled();
  });

  it("enables stream check button with providerId even without apiKey", async () => {
    vi.mocked(providerStreamCheck).mockResolvedValueOnce({
      ok: true,
      grade: "operational",
      duration_ms: 100,
      target_url: "https://example.com/v1/chat/completions",
      used_model: "gpt-4.1-mini",
      attempts: 1,
    });

    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://example.com",
            ping: { status: "idle" },
            streamCheck: { status: "idle" },
          },
        ]}
        cliKey="claude"
        providerId={1}
      />
    );

    const testBtn = screen.getByRole("button", { name: "测试" });
    expect(testBtn).not.toBeDisabled();

    fireEvent.click(testBtn);
    await waitFor(() => expect(screen.getByText("100ms")).toBeInTheDocument());
  });

  it("skips stream check for empty url or missing cliKey", () => {
    render(
      <TestWrapper
        initial={[{ id: "1", url: "", ping: { status: "idle" }, streamCheck: { status: "idle" } }]}
        apiKey="sk-test"
      />
    );

    const testBtn = screen.getByRole("button", { name: "测试" });
    expect(testBtn).toBeDisabled();
  });

  it("supports move up and move down buttons", () => {
    render(
      <TestWrapper
        initial={[
          { id: "1", url: "https://a", ping: { status: "idle" }, streamCheck: { status: "idle" } },
          { id: "2", url: "https://b", ping: { status: "idle" }, streamCheck: { status: "idle" } },
        ]}
      />
    );

    const inputs = screen.getAllByPlaceholderText("https://api.openai.com");
    expect(inputs).toHaveLength(2);

    // Move down first row
    const downButtons = screen.getAllByTitle("下移");
    fireEvent.click(downButtons[0]!);

    const updatedInputs = screen.getAllByPlaceholderText("https://api.openai.com");
    expect((updatedInputs[0] as HTMLInputElement).value).toBe("https://b");
    expect((updatedInputs[1] as HTMLInputElement).value).toBe("https://a");

    // Move up second row (which is now "https://a")
    const upButtons = screen.getAllByTitle("上移");
    fireEvent.click(upButtons[1]!);

    const finalInputs = screen.getAllByPlaceholderText("https://api.openai.com");
    expect((finalInputs[0] as HTMLInputElement).value).toBe("https://a");
    expect((finalInputs[1] as HTMLInputElement).value).toBe("https://b");
  });

  it("removes a row when clicking remove button", () => {
    render(
      <TestWrapper
        initial={[
          { id: "1", url: "https://a", ping: { status: "idle" }, streamCheck: { status: "idle" } },
          { id: "2", url: "https://b", ping: { status: "idle" }, streamCheck: { status: "idle" } },
        ]}
      />
    );

    const removeButtons = screen.getAllByRole("button", { name: "×" });
    fireEvent.click(removeButtons[0]!);

    const inputs = screen.getAllByPlaceholderText("https://api.openai.com");
    expect(inputs).toHaveLength(1);
    expect((inputs[0] as HTMLInputElement).value).toBe("https://b");
  });

  it("renders badges for pre-set stream check states", () => {
    render(
      <TestWrapper
        initial={[
          {
            id: "1",
            url: "https://a",
            ping: { status: "ok", ms: 50 },
            streamCheck: { status: "operational", ms: 200 },
          },
          {
            id: "2",
            url: "https://b",
            ping: { status: "idle" },
            streamCheck: { status: "degraded", ms: 4000 },
          },
          {
            id: "3",
            url: "https://c",
            ping: { status: "idle" },
            streamCheck: { status: "failed", message: "401", failureKind: "auth" },
          },
          {
            id: "4",
            url: "https://d",
            ping: { status: "idle" },
            streamCheck: { status: "checking" },
          },
        ]}
      />
    );

    expect(screen.getByText("50ms")).toBeInTheDocument();
    expect(screen.getByText("200ms")).toBeInTheDocument();
    expect(screen.getByText("4000ms")).toBeInTheDocument();
    expect(screen.getAllByText("失败").length).toBeGreaterThanOrEqual(1);
    // "checking" state shows "…"
    expect(screen.getAllByText("…").length).toBeGreaterThanOrEqual(1);
    // "checking" row shows "测试中…"
    expect(screen.getByRole("button", { name: "测试中…" })).toBeInTheDocument();
  });
});
