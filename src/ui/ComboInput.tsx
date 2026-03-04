import { useEffect, useMemo, useRef, useState } from "react";
import { ChevronDown } from "lucide-react";
import { Input } from "./Input";
import { cn } from "../utils/cn";
import { Popover, PopoverAnchor, PopoverContent } from "@/ui/shadcn/popover";

export type ComboInputProps = {
  value: string;
  onChange: (value: string) => void;
  options: string[];
  placeholder?: string;
  disabled?: boolean;
  className?: string;
};

export function ComboInput({
  value,
  onChange,
  options,
  placeholder,
  disabled,
  className,
}: ComboInputProps) {
  const [open, setOpen] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const filtered = useMemo(() => {
    const q = value.trim().toLowerCase();
    if (!q) return options.slice(0, 50);
    return options.filter((o) => o.toLowerCase().includes(q)).slice(0, 50);
  }, [options, value]);

  // Close dropdown when clicking outside
  useEffect(() => {
    if (!open) return;

    function handlePointerDown(e: PointerEvent) {
      const target = e.target as Node | null;
      // Keep open if clicking inside the input or the popover content
      if (inputRef.current?.parentElement?.contains(target)) return;
      // PopoverContent is portalled, check by data attribute
      const popoverContent = (target as Element | null)?.closest?.(
        "[data-radix-popper-content-wrapper]"
      );
      if (popoverContent) return;
      setOpen(false);
    }

    document.addEventListener("pointerdown", handlePointerDown);
    return () => document.removeEventListener("pointerdown", handlePointerDown);
  }, [open]);

  return (
    <Popover open={open} modal={false}>
      <PopoverAnchor asChild>
        <div className="relative">
          <Input
            ref={inputRef}
            value={value}
            onChange={(e) => {
              onChange(e.currentTarget.value);
              if (!open) setOpen(true);
            }}
            onFocus={() => setOpen(true)}
            placeholder={placeholder}
            disabled={disabled}
            className={cn("pr-9", className)}
            spellCheck={false}
            autoComplete="off"
          />

          <button
            type="button"
            className={cn(
              [
                "absolute right-1 top-1/2 -translate-y-1/2",
                "flex h-7 w-7 items-center justify-center rounded-md",
                "text-slate-400 hover:text-slate-600 dark:hover:text-slate-300",
                "transition-colors",
              ].join(" ")
            )}
            tabIndex={-1}
            disabled={disabled}
            aria-label={open ? "关闭选项列表" : "打开选项列表"}
            onClick={() => {
              setOpen((prev) => !prev);
              // Refocus input after toggling so user can keep typing
              inputRef.current?.focus();
            }}
          >
            <ChevronDown className="h-4 w-4" />
          </button>
        </div>
      </PopoverAnchor>

      <PopoverContent
        align="start"
        className="w-[var(--radix-popover-trigger-width)] p-1"
        onOpenAutoFocus={(e) => e.preventDefault()}
        onCloseAutoFocus={(e) => e.preventDefault()}
      >
        <div role="listbox" className="max-h-64 overflow-auto">
          {filtered.length === 0 ? (
            <div className="px-2 py-1.5 text-sm text-slate-500 dark:text-slate-400">无匹配项</div>
          ) : (
            filtered.map((opt) => (
              <button
                key={opt}
                type="button"
                role="option"
                className={cn(
                  [
                    "w-full rounded-md px-2 py-1.5 text-left text-sm",
                    "hover:bg-slate-100 dark:hover:bg-slate-700",
                  ].join(" ")
                )}
                onPointerDown={(e) => {
                  // Prevent input blur before onClick fires
                  e.preventDefault();
                }}
                onClick={() => {
                  onChange(opt);
                  setOpen(false);
                  inputRef.current?.focus();
                }}
              >
                <span className="font-mono">{opt}</span>
              </button>
            ))
          )}
        </div>
      </PopoverContent>
    </Popover>
  );
}
