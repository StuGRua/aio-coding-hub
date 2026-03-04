import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { ComboInput } from "../ComboInput";

describe("ui/ComboInput", () => {
  it("filters options and allows selecting an item", async () => {
    const onChange = vi.fn();

    function Harness() {
      const [value, setValue] = useState("");
      return (
        <ComboInput
          value={value}
          onChange={(next) => {
            onChange(next);
            setValue(next);
          }}
          placeholder="model"
          options={["gpt-4.1-mini", "gpt-4.1", "gemini-2.0-flash"]}
        />
      );
    }

    render(<Harness />);

    const input = screen.getByPlaceholderText("model");
    fireEvent.focus(input);
    expect(await screen.findByRole("option", { name: "gpt-4.1-mini" })).toBeInTheDocument();

    fireEvent.change(input, { target: { value: "4.1-mini" } });
    expect(await screen.findByRole("option", { name: "gpt-4.1-mini" })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: "gemini-2.0-flash" })).toBeNull();

    fireEvent.click(screen.getByRole("option", { name: "gpt-4.1-mini" }));
    expect(onChange).toHaveBeenLastCalledWith("gpt-4.1-mini");
  });

  it("toggles dropdown via chevron button", () => {
    function Harness() {
      const [value, setValue] = useState("");
      return (
        <ComboInput value={value} onChange={setValue} placeholder="model" options={["a", "b"]} />
      );
    }

    render(<Harness />);

    // Initially closed
    expect(screen.queryByRole("listbox")).toBeNull();

    // Click toggle button to open
    fireEvent.click(screen.getByLabelText("打开选项列表"));
    expect(screen.getByRole("listbox")).toBeInTheDocument();

    // Click toggle button to close
    fireEvent.click(screen.getByLabelText("关闭选项列表"));
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("shows no-match text when filter yields no results", async () => {
    function Harness() {
      const [value, setValue] = useState("zzz-nonexistent");
      return (
        <ComboInput
          value={value}
          onChange={setValue}
          placeholder="model"
          options={["gpt-4.1-mini", "gemini-2.0-flash"]}
        />
      );
    }

    render(<Harness />);

    const input = screen.getByPlaceholderText("model");
    fireEvent.focus(input);
    expect(await screen.findByText("无匹配项")).toBeInTheDocument();
  });

  it("opens dropdown on typing when closed", () => {
    function Harness() {
      const [value, setValue] = useState("");
      return (
        <ComboInput
          value={value}
          onChange={setValue}
          placeholder="model"
          options={["alpha", "beta"]}
        />
      );
    }

    render(<Harness />);

    expect(screen.queryByRole("listbox")).toBeNull();

    const input = screen.getByPlaceholderText("model");
    fireEvent.change(input, { target: { value: "a" } });
    expect(screen.getByRole("listbox")).toBeInTheDocument();
  });

  it("renders in disabled state", () => {
    function Harness() {
      return (
        <ComboInput value="" onChange={() => {}} placeholder="model" options={["a"]} disabled />
      );
    }

    render(<Harness />);
    expect(screen.getByPlaceholderText("model")).toBeDisabled();
    expect(screen.getByLabelText("打开选项列表")).toBeDisabled();
  });
});
