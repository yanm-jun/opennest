import { useEffect, useState } from "react";
import { OpenNestRecipeRegistry } from "./recipeRegistry";
import { getRecipeStatus, listRecipes, openRecipeDashboard, startRecipe, stopRecipe } from "./recipeRuntimeApi";
import { RecipeStatusBadge } from "./RecipeStatusBadge";
import type { OpenNestRecipeSummary, RecipeStatus } from "./types";

export function MyLibraryRecipes({ onOpenDetails }: { onOpenDetails: (appId: string) => void }) {
  const [apps, setApps] = useState<OpenNestRecipeSummary[]>(OpenNestRecipeRegistry);
  const [statuses, setStatuses] = useState<Record<string, RecipeStatus>>({});
  const [message, setMessage] = useState<string | null>(null);

  async function loadApps() {
    try {
      const loaded = await listRecipes();
      if (loaded.length) {
        setApps(loaded);
        return loaded;
      }
    } catch {
      // Browser preview / backend unavailable: keep bundled frontend registry as a safe fallback.
    }
    setApps(OpenNestRecipeRegistry);
    return OpenNestRecipeRegistry;
  }

  async function refresh() {
    const nextApps = await loadApps();
    const pairs = await Promise.all(
      nextApps.map(async (app) => [app.id, await getRecipeStatus(app.id)] as const),
    );
    setStatuses(Object.fromEntries(pairs));
  }

  async function runAction(label: string, action: () => Promise<{ ok?: boolean; error?: string; message?: string }>) {
    setMessage(null);
    try {
      const result = await action();
      if (result && result.ok === false) {
        setMessage(result.error ?? `${label} failed.`);
      } else {
        setMessage(result?.message ?? `${label} completed.`);
      }
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      await refresh().catch(() => undefined);
    }
  }

  useEffect(() => {
    refresh().catch(() => undefined);
  }, []);

  const installed = apps.filter((app) => statuses[app.id]?.installed);

  if (!installed.length) {
    return <div className="rounded-2xl border border-slate-200 bg-white p-6 text-sm text-slate-600">No recipe apps installed yet.</div>;
  }

  return (
    <section className="space-y-3">
      {message && <div className="rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 text-sm text-slate-700">{message}</div>}
      {installed.map((app) => {
        const status = statuses[app.id];
        return (
          <div key={app.id} className="flex items-center justify-between rounded-2xl border border-slate-200 bg-white p-4 shadow-sm">
            <div>
              <h3 className="font-semibold text-slate-950">{app.name}</h3>
              <p className="text-sm text-slate-500">{app.runtime}</p>
            </div>
            <div className="flex items-center gap-2">
              <RecipeStatusBadge state={status?.runState ?? "unknown"} />
              <button onClick={() => runAction("Start", () => startRecipe(app.id))} className="rounded-lg border px-3 py-1.5 text-sm">Start</button>
              <button onClick={() => runAction("Stop", () => stopRecipe(app.id))} className="rounded-lg border px-3 py-1.5 text-sm">Stop</button>
              <button onClick={() => runAction("Open", () => openRecipeDashboard(app.id))} className="rounded-lg border px-3 py-1.5 text-sm">Open</button>
              <button onClick={() => onOpenDetails(app.id)} className="rounded-lg bg-slate-950 px-3 py-1.5 text-sm text-white">Details</button>
            </div>
          </div>
        );
      })}
    </section>
  );
}
