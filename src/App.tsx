import { useEffect, useRef, useState } from "react";
import { AlertTriangle, ArrowRight, CheckCircle2, Copy, Loader2, Play } from "lucide-react";
import { FolderPicker } from "@/components/FolderPicker";
import { ResultList } from "@/components/ResultList";
import { cn } from "@/lib/cn";
import {
  compare as cmpInvoke,
  copyMissing,
  onCopyProgress,
  type CompareResult,
  type CopyProgress,
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
  const [progress, setProgress] = useState<CopyProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [successAt, setSuccessAt] = useState<number>(0);
  const copyCountRef = useRef(0);
  const successTimerRef = useRef<number | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    onCopyProgress((p) => setProgress(p)).then((u) => {
      unlisten = u;
    });
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const canCompare = !!allDir && !!sentDir && status === "idle";
  const canCopy =
    !!result && result.missing.length > 0 && !!outDir && status === "idle";

  async function runCompare() {
    if (!allDir || !sentDir) return;
    setStatus("comparing");
    setError(null);
    setCopyReport(null);
    setProgress(null);
    copyCountRef.current = 0;
    try {
      const r = await cmpInvoke(allDir, sentDir);
      setResult(r);
      if (r.extra.length > 0) {
        // 意料之外：已发送 ⊄ 全部
        window.alert(
          `⚠️ 检测到 ${r.extra.length} 个文件存在于「已发送」中，但在「全部」里找不到对应项。\n\n这通常意味着：\n· 选错了文件夹\n· 命名规则不一致\n· 已发送的内容来源不止一处\n\n请滚动到下方「多余清单」核对。`,
        );
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setStatus("idle");
    }
  }

  async function runCopy() {
    if (!result || !outDir) return;
    if (copyCountRef.current > 0) {
      const ok = window.confirm(
        "本次会话已经复制过一次。是否再次复制？\n\n（已存在的文件会被自动跳过）",
      );
      if (!ok) return;
    }

    setStatus("copying");
    setError(null);
    setCopyReport(null);
    setProgress({ done: 0, total: result.missing.length, current: "" });
    try {
      const paths = result.missing.map((m) => m.abs_path);
      const r = await copyMissing(paths, outDir);
      setCopyReport(r);
      copyCountRef.current += 1;
      setSuccessAt(Date.now());
      if (successTimerRef.current) window.clearTimeout(successTimerRef.current);
      successTimerRef.current = window.setTimeout(
        () => setSuccessAt(0),
        4000,
      ) as unknown as number;
    } catch (e) {
      setError(String(e));
    } finally {
      setStatus("idle");
    }
  }

  const stats = result?.stats;
  const pct =
    progress && progress.total > 0
      ? Math.round((progress.done / progress.total) * 100)
      : 0;

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
                <Stat
                  label="多余"
                  value={stats.extra_count}
                  highlight={stats.extra_count > 0}
                />
              </div>
            )}
          </div>

          {(status === "copying" || (progress && progress.done < progress.total)) &&
            progress && (
              <div className="rounded-md border border-zinc-200 bg-white px-3 py-2.5">
                <div className="mb-1.5 flex items-center justify-between font-mono text-xs">
                  <span className="truncate text-zinc-600" title={progress.current}>
                    {progress.current || "准备中…"}
                  </span>
                  <span className="ml-3 shrink-0 text-zinc-900">
                    {progress.done} / {progress.total} ({pct}%)
                  </span>
                </div>
                <div className="h-1.5 overflow-hidden rounded-full bg-zinc-100">
                  <div
                    className="h-full bg-zinc-900 transition-all duration-150"
                    style={{ width: `${pct}%` }}
                  />
                </div>
              </div>
            )}

          {successAt > 0 && copyReport && (
            <div className="flex items-center gap-2 rounded-md border border-zinc-900 bg-zinc-900 px-3 py-2 text-xs text-white">
              <CheckCircle2 size={14} />
              <span>
                复制成功！已复制 {copyReport.copied} 个文件
                {copyReport.skipped > 0 && `，跳过 ${copyReport.skipped} 个已存在`}
                {copyReport.failed.length > 0 &&
                  `，失败 ${copyReport.failed.length} 个`}
              </span>
            </div>
          )}

          {error && (
            <div className="rounded-md border border-zinc-900 bg-zinc-900 px-3 py-2 text-xs text-white">
              错误：{error}
            </div>
          )}
        </section>

        <section className="grid min-h-0 grid-rows-[1fr_auto] gap-4">
          <ResultList
            title="缺失清单"
            count={result?.missing.length ?? 0}
            entries={result?.missing ?? []}
            emptyText={result ? "无缺失" : "尚未对比"}
          />

          {result && result.extra.length > 0 && (
            <div className="rounded-lg border-2 border-zinc-900 bg-white shadow-[0_-2px_0_0_rgba(0,0,0,0.04)]">
              <div className="flex items-center gap-2 border-b-2 border-zinc-900 bg-zinc-900 px-3 py-2 text-white">
                <AlertTriangle size={14} className="shrink-0" />
                <span className="text-sm font-semibold">
                  ⚠ 多余清单 — 已发送中存在「全部」里没有的文件
                </span>
                <span className="ml-auto rounded bg-white px-1.5 py-0.5 font-mono text-xs text-zinc-900">
                  {result.extra.length}
                </span>
              </div>
              <div className="px-3 py-2 text-xs text-zinc-700">
                这通常是意料之外的情况：可能选错了文件夹，或这些文件来自其他来源。请逐一核对：
              </div>
              <div className="max-h-48 overflow-auto border-t border-zinc-200">
                {result.extra.map((e) => (
                  <div
                    key={e.abs_path}
                    className="flex items-center gap-3 border-b border-zinc-100 px-3 py-1.5 font-mono text-xs last:border-b-0"
                    title={e.abs_path}
                  >
                    <span className="shrink-0 rounded bg-zinc-900 px-1.5 py-0.5 text-[10px] text-white">
                      {e.key}
                    </span>
                    <span className="truncate text-zinc-700">{e.full_name}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
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
      <span className={highlight ? "font-semibold text-zinc-900" : "text-zinc-700"}>
        {value}
      </span>
    </span>
  );
}
