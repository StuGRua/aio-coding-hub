import { useMemo, useRef, useState } from "react";
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
  const listRef = useRef<HTMLDivElement>(null);
  const ignoreNextFocusOpenRef = useRef(false);

  const filtered = useMemo(() => {
    const q = value.trim().toLowerCase();
    if (!q) return options.slice(0, 50);
    return options.filter((o) => o.toLowerCase().includes(q)).slice(0, 50);
  }, [options, value]);

  function scrollListBy(deltaY: number) {
    const el = listRef.current;
    if (!el) return false;
    if (el.scrollHeight <= el.clientHeight) return false;
    const prev = el.scrollTop;
    const max = Math.max(0, el.scrollHeight - el.clientHeight);
    const next = Math.max(0, Math.min(prev + deltaY, max));
    if (next === prev) return false;
    el.scrollTop = next;
    return true;
  }

  return (
    <Popover open={open} onOpenChange={setOpen} modal={false}>
      <PopoverAnchor asChild>
        <div className="relative">
          <Input
            ref={inputRef}
            value={value}
            onChange={(e) => {
              onChange(e.currentTarget.value);
              if (!open) setOpen(true);
            }}
            onFocus={() => {
              if (ignoreNextFocusOpenRef.current) {
                ignoreNextFocusOpenRef.current = false;
                return;
              }
              setOpen(true);
            }}
            onWheel={(e) => {
              if (!open) return;
              const scrolled = scrollListBy(e.deltaY);
              if (!scrolled) return;
              // When the cursor is on the input, redirect wheel to the list and prevent page scroll.
              e.preventDefault();
            }}
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
            onPointerDown={(e) => {
              // Keep focus on input; avoids focus/blur-driven open/close glitches.
              e.preventDefault();
            }}
            onClick={() => {
              setOpen((prev) => {
                const next = !prev;
                if (next) {
                  ignoreNextFocusOpenRef.current = true;
                  inputRef.current?.focus();
                }
                return next;
              });
            }}
          >
            <ChevronDown className="h-4 w-4" />
          </button>
        </div>
      </PopoverAnchor>

      <PopoverContent
        align="start"
        collisionPadding={12}
        className="z-[60] w-auto min-w-[var(--radix-popper-anchor-width)] max-w-[90vw] p-1"
        onOpenAutoFocus={(e) => e.preventDefault()}
        onCloseAutoFocus={(e) => e.preventDefault()}
      >
        <div
          ref={listRef}
          role="listbox"
          className="max-h-64 overflow-y-auto overscroll-contain"
          onWheel={(e) => {
            // Keep wheel scrolling inside the list; avoid bubbling to parent scroll containers.
            e.stopPropagation();
          }}
        >
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
                  ignoreNextFocusOpenRef.current = true;
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
