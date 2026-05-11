import {
  AppWindow,
  BookMarked,
  LibraryBig,
  Logs,
  Settings,
  Sparkles,
  Wrench,
} from "lucide-react";
import { RuntimeGuardCard } from "./RuntimeGuardCard";
import type { RuntimeGuardItem } from "../types";

const sidebarItems = [
  { id: "app-center", label: "App Center", icon: AppWindow, active: true },
  { id: "my-library", label: "My Library", icon: LibraryBig },
  { id: "recipes", label: "Recipes", icon: BookMarked },
  { id: "runtimes", label: "Runtimes", icon: Wrench },
  { id: "logs", label: "Logs", icon: Logs },
  { id: "settings", label: "Settings", icon: Settings },
];

export function AppSidebar({ guardItems }: { guardItems: RuntimeGuardItem[] }) {
  return (
    <aside className="flex h-full flex-col rounded-[28px] border border-slate-200 bg-[#fdfdfc] p-5 shadow-sm">
      <div className="flex items-center gap-3 rounded-3xl border border-slate-200 bg-white px-4 py-3">
        <div className="flex h-11 w-11 items-center justify-center rounded-2xl bg-orange-100 text-orange-700">
          <Sparkles className="h-5 w-5" />
        </div>
        <div>
          <div className="text-sm font-semibold text-slate-950">OpenNest</div>
          <div className="text-xs text-slate-500">AI App Runtime</div>
        </div>
      </div>

      <nav className="mt-6 space-y-2">
        {sidebarItems.map((item) => {
          const Icon = item.icon;
          return (
            <button
              key={item.id}
              type="button"
              className={`flex w-full items-center gap-3 rounded-2xl px-4 py-3 text-left text-sm transition ${
                item.active
                  ? "bg-slate-900 text-white shadow-sm"
                  : "text-slate-600 hover:bg-white hover:text-slate-950"
              }`}
            >
              <Icon className="h-4 w-4" />
              <span className="font-medium">{item.label}</span>
            </button>
          );
        })}
      </nav>

      <div className="mt-auto pt-6">
        <RuntimeGuardCard items={guardItems} />
      </div>
    </aside>
  );
}
