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
});
