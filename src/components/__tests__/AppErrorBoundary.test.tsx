import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { AppErrorBoundary } from "../AppErrorBoundary";
import { reportRenderError } from "../../services/frontendErrorReporter";

vi.mock("../../services/frontendErrorReporter", () => ({
  reportRenderError: vi.fn(),
}));

// A component that throws during render
function ThrowingChild({ message }: { message: string }): React.JSX.Element {
  throw new Error(message);
}

// A component that renders normally
function GoodChild() {
  return <div>healthy child</div>;
}

describe("components/AppErrorBoundary", () => {
  it("renders children when no error occurs", () => {
    render(
      <AppErrorBoundary>
        <GoodChild />
      </AppErrorBoundary>
    );

    expect(screen.getByText("healthy child")).toBeInTheDocument();
  });

  it("renders fallback UI when a child throws during render", () => {
    // Suppress React error boundary console.error noise
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(
      <AppErrorBoundary>
        <ThrowingChild message="boom" />
      </AppErrorBoundary>
    );

    // Fallback heading
    expect(screen.getByText("页面渲染异常")).toBeInTheDocument();
    // Fallback description
    expect(screen.getByText(/已记录错误日志/)).toBeInTheDocument();

    spy.mockRestore();
  });

  it("calls reportRenderError with the caught error and componentStack", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(
      <AppErrorBoundary>
        <ThrowingChild message="report test" />
      </AppErrorBoundary>
    );

    expect(reportRenderError).toHaveBeenCalledTimes(1);
    expect(reportRenderError).toHaveBeenCalledWith(
      expect.objectContaining({ message: "report test" }),
      expect.objectContaining({ componentStack: expect.any(String) })
    );

    spy.mockRestore();
  });

  it("does not show fallback UI for children that render successfully", () => {
    render(
      <AppErrorBoundary>
        <div>first</div>
        <div>second</div>
      </AppErrorBoundary>
    );

    expect(screen.getByText("first")).toBeInTheDocument();
    expect(screen.getByText("second")).toBeInTheDocument();
    expect(screen.queryByText("页面渲染异常")).not.toBeInTheDocument();
  });

  it("outer boundary catches error when inner boundary is absent", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(
      <AppErrorBoundary>
        <div>
          <ThrowingChild message="nested boom" />
        </div>
      </AppErrorBoundary>
    );

    expect(screen.getByText("页面渲染异常")).toBeInTheDocument();
    expect(reportRenderError).toHaveBeenCalledWith(
      expect.objectContaining({ message: "nested boom" }),
      expect.any(Object)
    );

    spy.mockRestore();
  });

  it("inner boundary catches error before outer boundary", () => {
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(
      <AppErrorBoundary>
        <div>outer content</div>
        <AppErrorBoundary>
          <ThrowingChild message="inner boom" />
        </AppErrorBoundary>
      </AppErrorBoundary>
    );

    // Inner boundary shows its own fallback
    expect(screen.getByText("页面渲染异常")).toBeInTheDocument();
    // Outer content is still rendered
    expect(screen.getByText("outer content")).toBeInTheDocument();

    spy.mockRestore();
  });

  it("shows reload action in fallback UI", () => {
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    render(
      <AppErrorBoundary>
        <ThrowingChild message="reload boom" />
      </AppErrorBoundary>
    );

    expect(screen.getByRole("button", { name: "重新加载" })).toBeInTheDocument();
    consoleSpy.mockRestore();
  });
});
