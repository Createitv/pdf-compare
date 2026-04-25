import { Folder, FolderOpen } from "lucide-react";
import { cn } from "@/lib/cn";
import { pickDirectory } from "@/lib/tauri";

interface Props {
  index: number;
  label: string;
  value: string | null;
  onChange: (path: string) => void;
  disabled?: boolean;
  highlight?: boolean;
}

export function FolderPicker({
  index,
  label,
  value,
  onChange,
  disabled,
  highlight,
}: Props) {
  async function pick() {
    const p = await pickDirectory();
    if (p) onChange(p);
  }

  return (
    <div
      className={cn(
        "flex items-center gap-3 rounded-lg border bg-white p-3 transition",
        highlight
          ? "animate-pulse border-zinc-900 ring-2 ring-zinc-900 ring-offset-2"
          : "border-zinc-200",
      )}
    >
      <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md bg-zinc-900 text-white">
        <span className="font-mono text-sm">{index}</span>
      </div>
      <div className="min-w-0 flex-1">
        <div className="text-xs uppercase tracking-wide text-zinc-500">{label}</div>
        <div
          className={cn(
            "mt-0.5 truncate font-mono text-sm",
            value ? "text-zinc-900" : "text-zinc-400",
          )}
          title={value ?? ""}
        >
          {value || "未选择"}
        </div>
      </div>
      <button
        onClick={pick}
        disabled={disabled}
        className={cn(
          "inline-flex items-center gap-2 rounded-md border border-zinc-300 bg-white px-3 py-1.5 text-sm font-medium text-zinc-900 transition",
          "hover:border-zinc-900 hover:shadow-sm active:scale-[0.98]",
          "disabled:cursor-not-allowed disabled:opacity-40",
        )}
      >
        {value ? <FolderOpen size={14} /> : <Folder size={14} />}
        选择
      </button>
    </div>
  );
}
