import { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { FileEntry } from "@/lib/tauri";

interface Props {
  title: string;
  count: number;
  entries: FileEntry[];
  emptyText: string;
  tone?: "neutral" | "warn";
}

export function ResultList({ title, count, entries, emptyText, tone = "neutral" }: Props) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virt = useVirtualizer({
    count: entries.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 32,
    overscan: 12,
  });

  return (
    <div className="flex min-h-0 flex-1 flex-col rounded-lg border border-zinc-200 bg-white">
      <div className="flex items-center justify-between border-b border-zinc-200 px-3 py-2">
        <span className="text-sm font-semibold text-zinc-900">{title}</span>
        <span
          className={
            tone === "warn"
              ? "rounded bg-zinc-900 px-1.5 py-0.5 font-mono text-xs text-white"
              : "rounded bg-zinc-100 px-1.5 py-0.5 font-mono text-xs text-zinc-700"
          }
        >
          {count}
        </span>
      </div>
      {entries.length === 0 ? (
        <div className="flex flex-1 items-center justify-center text-sm text-zinc-400">
          {emptyText}
        </div>
      ) : (
        <div ref={parentRef} className="min-h-0 flex-1 overflow-auto">
          <div style={{ height: virt.getTotalSize(), position: "relative" }}>
            {virt.getVirtualItems().map((vi) => {
              const e = entries[vi.index];
              return (
                <div
                  key={vi.key}
                  className="absolute inset-x-0 flex items-center gap-3 border-b border-zinc-100 px-3 font-mono text-xs"
                  style={{ top: vi.start, height: vi.size }}
                  title={e.abs_path}
                >
                  <span className="shrink-0 rounded bg-zinc-900 px-1.5 py-0.5 text-[10px] text-white">
                    {e.key}
                  </span>
                  <span className="truncate text-zinc-700">{e.full_name}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
