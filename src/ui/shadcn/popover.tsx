import * as PopoverPrimitive from "@radix-ui/react-popover";
import { forwardRef } from "react";
import { cn } from "@/ui/shadcn/utils";

export const Popover = PopoverPrimitive.Root;
export const PopoverTrigger = PopoverPrimitive.Trigger;
export const PopoverAnchor = PopoverPrimitive.Anchor;

export const PopoverContent = forwardRef<
  React.ElementRef<typeof PopoverPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof PopoverPrimitive.Content>
>(function PopoverContent({ className, align = "end", sideOffset = 8, ...props }, ref) {
  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Content
        ref={ref}
        align={align}
        sideOffset={sideOffset}
        className={cn(
          [
            "z-50 rounded-lg border border-slate-200 bg-white p-2 shadow-lg outline-none",
            "dark:border-slate-700 dark:bg-slate-800 dark:shadow-slate-900/50",
          ].join(" "),
          className
        )}
        {...props}
      />
    </PopoverPrimitive.Portal>
  );
});
