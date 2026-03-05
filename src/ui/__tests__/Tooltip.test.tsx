import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import { Tooltip } from "../Tooltip";

describe("ui/Tooltip", () => {
  it("renders children without tooltip content initially", () => {
    render(
      <Tooltip content="Tip text">
        <span>Hover me</span>
      </Tooltip>
    );
    expect(screen.getByText("Hover me")).toBeInTheDocument();
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  });

  it("shows tooltip on mouseEnter and hides on mouseLeave", async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Hello tooltip">
        <span>Anchor</span>
      </Tooltip>
    );

    const anchor = screen.getByText("Anchor");
    await user.hover(anchor);
    expect(await screen.findByRole("tooltip")).toHaveTextContent("Hello tooltip");

    await user.unhover(anchor);
    await waitFor(() => expect(screen.queryByRole("tooltip")).not.toBeInTheDocument());
  });

  it("renders with placement=top by default", async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Top tip">
        <span>Anchor</span>
      </Tooltip>
    );

    await user.hover(screen.getByText("Anchor"));
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("Top tip");
    const container = tooltip.closest("[data-side]");
    expect(container).not.toBeNull();
    expect(container).toHaveAttribute("data-side", "top");
  });

  it("renders with placement=bottom", async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Bottom tip" placement="bottom">
        <span>Anchor</span>
      </Tooltip>
    );

    await user.hover(screen.getByText("Anchor"));
    const tooltip = await screen.findByRole("tooltip");
    expect(tooltip).toHaveTextContent("Bottom tip");
    const container = tooltip.closest("[data-side]");
    expect(container).not.toBeNull();
    expect(container).toHaveAttribute("data-side", "bottom");
  });

  it("merges custom className on the anchor wrapper", () => {
    const { container } = render(
      <Tooltip content="Tip" className="anchor-class">
        <span>Anchor</span>
      </Tooltip>
    );
    expect(container.querySelector(".anchor-class")).toBeInTheDocument();
  });

  it("merges contentClassName on the tooltip content", async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Styled tip" contentClassName="tip-style">
        <span>Anchor</span>
      </Tooltip>
    );

    await user.hover(screen.getByText("Anchor"));
    await waitFor(() =>
      expect(screen.getByRole("tooltip").closest(".tip-style")).toBeInTheDocument()
    );
  });

  it("wraps non-element children in span", async () => {
    const user = userEvent.setup();
    render(
      <Tooltip content="Text tip" className="wrapper-class">
        Plain text child
      </Tooltip>
    );

    const wrapper = screen.getByText("Plain text child").closest(".wrapper-class");
    expect(wrapper).toBeInTheDocument();
    expect(wrapper?.tagName).toBe("SPAN");

    await user.hover(screen.getByText("Plain text child"));
    expect(await screen.findByRole("tooltip")).toHaveTextContent("Text tip");
  });
});
