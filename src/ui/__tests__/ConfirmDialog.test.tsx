import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ConfirmDialog } from "../ConfirmDialog";

describe("ui/ConfirmDialog", () => {
  it("calls onClose when dialog close button is clicked", async () => {
    const onClose = vi.fn();
    const onConfirm = vi.fn();

    render(
      <ConfirmDialog
        open={true}
        title="确认操作"
        onClose={onClose}
        onConfirm={onConfirm}
        confirmLabel="确认"
        confirmingLabel="处理中..."
        confirming={false}
      />
    );

    const closeButton = screen.getByRole("button", { name: "关闭" });
    await userEvent.click(closeButton);

    expect(onClose).toHaveBeenCalled();
  });

  it("calls onConfirm when confirm button is clicked", async () => {
    const onClose = vi.fn();
    const onConfirm = vi.fn();

    render(
      <ConfirmDialog
        open={true}
        title="确认操作"
        onClose={onClose}
        onConfirm={onConfirm}
        confirmLabel="确认删除"
        confirmingLabel="处理中..."
        confirming={false}
      />
    );

    await userEvent.click(screen.getByRole("button", { name: "确认删除" }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("disables confirm button and shows confirming label while confirming", async () => {
    const onConfirm = vi.fn();

    render(
      <ConfirmDialog
        open={true}
        title="确认操作"
        onClose={vi.fn()}
        onConfirm={onConfirm}
        confirmLabel="确认"
        confirmingLabel="处理中..."
        confirming={true}
      />
    );

    const confirmButton = screen.getByRole("button", { name: "处理中..." });
    expect(confirmButton).toBeDisabled();
    await userEvent.click(confirmButton);
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("disables confirm button when disabled prop is true", () => {
    render(
      <ConfirmDialog
        open={true}
        title="确认操作"
        onClose={vi.fn()}
        onConfirm={vi.fn()}
        confirmLabel="确认"
        confirmingLabel="处理中..."
        confirming={false}
        disabled={true}
      />
    );

    expect(screen.getByRole("button", { name: "确认" })).toBeDisabled();
  });

  it("uses confirmVariant styles for confirm button", () => {
    render(
      <ConfirmDialog
        open={true}
        title="确认操作"
        onClose={vi.fn()}
        onConfirm={vi.fn()}
        confirmLabel="确认"
        confirmingLabel="处理中..."
        confirming={false}
        confirmVariant="danger"
      />
    );

    expect(screen.getByRole("button", { name: "确认" })).toHaveClass("text-rose-700");
  });
});
