import { useMemo, useState } from "react";
import { ArrowRight, Copy, Loader2, Play } from "lucide-react";
import { FolderPicker } from "@/components/FolderPicker";
import { ResultList } from "@/components/ResultList";
import { cn } from "@/lib/cn";
import {
  compare as cmpInvoke,
  copyMissing,
  type CompareResult,
  type CopyReport,
} from "@/lib/tauri";

type Status = "idle" | "comparing" | "copying";

export default function App() {
  const [allDir, setAllDir] = useState<string | null>(null);
  const [sentDir, setSentDir] = useState<string | null>(null);
  const [outDir, setOutDir] = useState<string | null>(null);
  const [status, setStatus] = useState<Status>("idle");
  const [result, setResult] = useState<CompareResult | null>(null);
  const [copyReport, setCopyReport] = useState<CopyReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  const canCompare = !!allDir && !!sentDir && status === "idle";
  const canCopy = !!result && result.missing.length > 0 && !!outDir && status === "idle";

  async function runCompare() {
    if (!allDir || !sentDir) return;
    setStatus("comparing");
    setError(null);
    setCopyReport(null);
    try {
      const r = await cmpInvoke(allDir, sentDir);
      setResult(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setStatus("idle");
    }
  }

  async function runCopy() {
    if (!result || !outDir) return;
    setStatus("copying");
    setError(null);
    try {
      const paths = result.missing.map((m) => m.abs_path);
      const r = await copyMissing(paths, outDir);
      setCopyReport(r);
    } catch (e) {
      setError(String(e));
    } finally {
      setStatus("idle");
    }
  }

  const stats = useMemo(() => result?.stats, [result]);

  return (
    <div className="flex h-screen flex-col bg-zinc-50">
      <header className="flex items-center justify-between border-b border-zinc-200 bg-white px-5 py-3">
        <div className="flex items-center gap-2">
          <div className="h-6 w-6 rounded bg-zinc-900" />
          <span className="font-semibold tracking-tight">PDF Compare</span>
        </div>
        <span className="font-mono text-xs text-zinc-400">v0.1.0</span>
      </header>

      <main className="grid min-h-0 flex-1 grid-rows-[auto_1fr] gap-4 p-5">
        <section className="grid gap-3">
          <div className="grid gap-3 md:grid-cols-3">
            <FolderPicker
              index={1}
              label="全部文件夹"
              value={allDir}
              onChange={setAllDir}
              disabled={status !== "idle"}
            />
            <FolderPicker
              index={2}
              label="已发送文件夹"
              value={sentDir}
              onChange={setSentDir}
              disabled={status !== "idle"}
            />
            <FolderPicker
              index={3}
              label="输出文件夹"
              value={outDir}
              onChange={setOutDir}
              disabled={status !== "idle"}
            />
          </div>

          <div className="flex flex-wrap items-center gap-3">
            <button
              onClick={runCompare}
              disabled={!canCompare}
              className={cn(
                "inline-flex items-center gap-2 rounded-md bg-zinc-900 px-4 py-2 text-sm font-medium text-white transition",
                "hover:bg-black active:scale-[0.98]",
                "disabled:cursor-not-allowed disabled:bg-zinc-300",
              )}
            >
              {status === "comparing" ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <Play size={14} />
              )}
              开始对比
            </button>

            <button
              onClick={runCopy}
              disabled={!canCopy}
              className={cn(
                "inline-flex items-center gap-2 rounded-md border border-zinc-900 bg-white px-4 py-2 text-sm font-medium text-zinc-900 transition",
                "hover:bg-zinc-900 hover:text-white active:scale-[0.98]",
                "disabled:cursor-not-allowed disabled:border-zinc-200 disabled:bg-white disabled:text-zinc-300",
              )}
            >
              {status === "copying" ? (
                <Loader2 size={14} className="animate-spin" />
              ) : (
                <Copy size={14} />
              )}
              复制缺失文件到输出
              <ArrowRight size={14} />
            </button>

            {stats && (
              <div className="ml-auto flex items-center gap-4 font-mono text-xs text-zinc-600">
                <Stat label="左" value={stats.total_left} />
                <Stat label="右" value={stats.total_right} />
                <Stat label="缺失" value={stats.missing_count} highlight />
                <Stat label="异常" value={stats.anomaly_count} highlight={stats.anomaly_count > 0} />
              </div>
            )}
          </div>

          {error && (
            <div className="rounded-md border border-zinc-900 bg-zinc-900 px-3 py-2 text-xs text-white">
              错误：{error}
            </div>
          )}
          {copyReport && (
            <div className="rounded-md border border-zinc-200 bg-white px-3 py-2 font-mono text-xs text-zinc-700">
              已复制 {copyReport.copied} 个文件
              {copyReport.failed.length > 0 && `，${copyReport.failed.length} 个失败`}
            </div>
          )}
        </section>

        <section className="grid min-h-0 grid-cols-1 gap-4 md:grid-cols-2">
          <ResultList
            title="缺失清单"
            count={result?.missing.length ?? 0}
            entries={result?.missing ?? []}
            emptyText={result ? "无缺失" : "尚未对比"}
          />
          <ResultList
            title="异常清单（前缀完全无对应）"
            count={result?.anomalies.length ?? 0}
            entries={result?.anomalies ?? []}
            emptyText={result ? "无异常" : "尚未对比"}
            tone="warn"
          />
        </section>
      </main>
    </div>
  );
}

function Stat({
  label,
  value,
  highlight,
}: {
  label: string;
  value: number;
  highlight?: boolean;
}) {
  return (
    <span className="inline-flex items-center gap-1.5">
      <span className="text-zinc-400">{label}</span>
      <span className={highlight ? "font-semibold text-zinc-900" : "text-zinc-700"}>{value}</span>
    </span>
  );
}
