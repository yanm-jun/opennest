import { ShieldCheck } from "lucide-react";
import type { RuntimeGuardItem } from "../types";

export function RuntimeGuardCard({ items }: { items: RuntimeGuardItem[] }) {
  return (
    <section className="rounded-3xl border border-slate-200 bg-white p-4 shadow-sm">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-2xl bg-emerald-50 text-emerald-700">
          <ShieldCheck className="h-5 w-5" />
        </div>
        <div>
          <h3 className="text-sm font-semibold text-slate-950">Runtime Guard</h3>
          <p className="text-xs text-slate-500">Desktop runtime protections surfaced before install.</p>
        </div>
      </div>

      <div className="mt-4 space-y-3">
        {items.map((item) => (
          <div key={item.label} className="rounded-2xl bg-slate-50 px-3 py-2.5">
            <div className="flex items-center justify-between gap-3">
              <span className="text-sm font-medium text-slate-800">{item.label}</span>
              <span className={`rounded-full px-2 py-0.5 text-[11px] font-medium ${item.healthy ? "bg-emerald-100 text-emerald-700" : "bg-amber-100 text-amber-700"}`}>
                {item.healthy ? "On" : "Check"}
              </span>
            </div>
            <p className="mt-1 text-xs text-slate-500">{item.value}</p>
          </div>
        ))}
      </div>
    </section>
  );
}
