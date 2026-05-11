import {
  Activity,
  ArchiveRestore,
  Play,
  RotateCcw,
  ScrollText,
  ShieldEllipsis,
  Square,
  Trash2,
} from "lucide-react";

const controls = [
  { id: "check", label: "Check", icon: Activity, tone: "default" },
  { id: "start", label: "Start", icon: Play, tone: "primary" },
  { id: "stop", label: "Stop", icon: Square, tone: "default" },
  { id: "restart", label: "Restart", icon: RotateCcw, tone: "default" },
  { id: "logs", label: "Logs", icon: ScrollText, tone: "default" },
  { id: "rollback", label: "Rollback", icon: ArchiveRestore, tone: "default" },
  { id: "keep-data", label: "Keep data", icon: ShieldEllipsis, tone: "default" },
  { id: "remove-data", label: "Remove data", icon: Trash2, tone: "danger" },
] as const;

export function RuntimeControls({
  onAction,
}: {
  onAction: (action: (typeof controls)[number]["id"]) => void;
}) {
  return (
    <section className="rounded-[28px] border border-slate-200 bg-white p-6 shadow-sm">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-lg font-semibold text-slate-950">Runtime Controls</h3>
          <p className="mt-1 text-sm text-slate-600">UI preview for recipe runtime actions. Hook these buttons to backend commands later.</p>
        </div>
      </div>

      <div className="mt-5 grid gap-3 md:grid-cols-4">
        {controls.map((control) => {
          const Icon = control.icon;
          return (
            <button
              key={control.id}
              type="button"
              onClick={() => onAction(control.id)}
              className={`flex items-center justify-center gap-2 rounded-2xl border px-4 py-3 text-sm font-medium transition ${
                control.tone === "primary"
                  ? "border-slate-900 bg-slate-900 text-white"
                  : control.tone === "danger"
                    ? "border-rose-200 bg-rose-50 text-rose-700"
                    : "border-slate-200 bg-white text-slate-800 hover:bg-slate-50"
              }`}
            >
              <Icon className="h-4 w-4" />
              {control.label}
            </button>
          );
        })}
      </div>
    </section>
  );
}
