import { useEffect, useMemo, useState } from "react";
import { OpenNestRecipeRegistry } from "./recipeRegistry";
import { listRecipes } from "./recipeRuntimeApi";
import type { OpenNestRecipeSummary } from "./types";
import { translateRecipeText, translateRuntimeKind, useI18n } from "../../i18n";

export function RecipeAppCenter({ onOpenApp }: { onOpenApp: (appId: string) => void }) {
  const { lang, t } = useI18n();
  const [apps, setApps] = useState<OpenNestRecipeSummary[]>(OpenNestRecipeRegistry);
  const [showImport, setShowImport] = useState(false);
  const [search, setSearch] = useState("");
  const [filterCategory, setFilterCategory] = useState<string | null>(null);
  const [showMarketplace, setShowMarketplace] = useState(false);
  const marketplaceApps: any[] = [];
  const marketLoading = false;
  const [importJson, setImportJson] /* import disabled */ = useState('');
  const [importError, setImportError] = useState<string | null>(null);
  const [importSuccess, setImportSuccess] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  async function refreshApps() {
    try {
      const loaded = await listRecipes();
      if (loaded.length > 0) setApps(loaded);
    } catch (error) {
      setLoadError(error instanceof Error ? error.message : String(error));
    }
  }

  useEffect(() => { refreshApps(); }, []);

  async function handleImport() /* disabled - marketplace stub */ {
    setImportError(null);
    setImportSuccess(null);
    try {
      throw new Error("Import not available in this build.");
      /* import success stub */
      setImportJson('');
      await refreshApps();
    } catch (error: any) {
      setImportError(error?.message || String(error));
    }
  }

  const categories = useMemo(() => [...new Set(apps.map((a) => a.category))], [apps]);
  const featured = useMemo(() => apps.filter((app) => {
    if (!app.featured) return false;
    if (search && ![app.name, app.summary, app.category].join(" ").toLowerCase().includes(search.toLowerCase())) return false;
    if (filterCategory && app.category !== filterCategory) return false;
    return true;
  }), [apps, search, filterCategory]);
  const standard = useMemo(() => apps.filter((app) => {
    if (app.featured) return false;
    if (search && ![app.name, app.summary, app.category].join(" ").toLowerCase().includes(search.toLowerCase())) return false;
    if (filterCategory && app.category !== filterCategory) return false;
    return true;
  }), [apps, search, filterCategory]);

  return (
    <section className="grid gap-4">
      {/* Compact toolbar */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-[20px] font-semibold">应用</h2>
          {loadError && <p className="text-[11px] text-[var(--fg-soft)]">使用内置元数据（后端离线）</p>}
        </div>
        <button
          onClick={() => setShowImport(!showImport)}
          className="rounded-lg border border-[var(--border-soft)] px-3 py-1.5 text-[12px] font-medium text-[var(--fg-soft)] hover:text-[var(--fg-strong)]"
        >
          {showImport ? '关闭' : '+ 导入 JSON'}
        </button>
      </div>

      {/* Import panel */}
      {showImport && (
        <div className="surface-card space-y-3 p-4">
          <textarea
            className="w-full h-28 rounded-lg border border-[var(--border-soft)] bg-[var(--bg-strong)] p-3 text-[13px] font-mono text-[var(--fg-strong)] resize-y"
            value={importJson}
            onChange={(e) => setImportJson(e.target.value)}
            placeholder='{ "id": "my-app", "name": "My App", "runtime": "webview", "dashboard": { "url": "http://localhost:8080" } }'
          />
          {importError && <div className="text-[12px] text-red-600">{importError}</div>}
          {importSuccess && <div className="text-[12px] text-green-600">已导入: {importSuccess}</div>}
          <div className="flex gap-2">
            <button onClick={handleImport} className="rounded-lg bg-[var(--fg-strong)] px-4 py-1.5 text-[12px] font-medium text-[var(--bg-strong)]">安装</button>
            <button onClick={() => { setShowImport(false); setImportError(null); }} className="rounded-lg border border-[var(--border-soft)] px-4 py-1.5 text-[12px] font-medium text-[var(--fg-soft)]">取消</button>
          </div>
        </div>
      )}

      {/* Featured apps */}
      {featured.length > 0 && (
        <>
          <h3 className="text-[13px] font-medium uppercase tracking-wide text-[var(--fg-muted)]">精选</h3>
          <div className="grid gap-3 sm:grid-cols-2">
            {featured.map((app) => (
              <FeaturedCard key={app.id} app={app} onClick={() => onOpenApp(app.id)} />
            ))}
          </div>
        </>
      )}

      {/* Marketplace */}
      {showMarketplace && (
        <div className="surface-card space-y-3 p-4">
          <h3 className="text-[14px] font-medium">应用市场</h3>
          {marketLoading && <p className="text-[12px] text-[var(--fg-soft)]">加载中...</p>}
          {marketplaceApps.length === 0 && !marketLoading && <p className="text-[12px] text-[var(--fg-soft)]">暂无可用的社区应用，请稍后重试</p>}
          <div className="grid gap-2 sm:grid-cols-2">
            {marketplaceApps.map((app: any) => (
              <div key={app.id} className="flex items-center justify-between rounded-lg border border-[var(--border-soft)] p-3">
                <div className="min-w-0">
                  <h4 className="text-[13px] font-medium truncate">{app.name}</h4>
                  <p className="text-[11px] text-[var(--fg-soft)] truncate">{app.summary || app.category}</p>
                </div>
                <button
                  onClick={async () => {
                    try {
                      await /* importUserRecipe stub */ (async () => { throw new Error("stub"); })() as any // JSON.stringify(app));
                      setImportSuccess(app.name || app.id);
                      await refreshApps();
                    } catch (e: any) {
                      setImportError(e?.message || String(e));
                    }
                  }}
                  className="shrink-0 rounded-md bg-[var(--fg-strong)] px-3 py-1 text-[11px] font-medium text-[var(--bg-strong)] hover:opacity-90"
                >
                  导入
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* All apps */}
      <h3 className="text-[13px] font-medium uppercase tracking-wide text-[var(--fg-muted)]">全部应用</h3>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {standard.map((app) => (
          <StandardCard key={app.id} app={app} onClick={() => onOpenApp(app.id)} />
        ))}
      </div>
    </section>
  );
}

function FeaturedCard({ app, onClick }: { app: OpenNestRecipeSummary; onClick: () => void }) {
  const { lang, t } = useI18n();
  const recipe = translateRecipeText(app, lang);
  return (
    <button onClick={onClick} className="surface-card group flex flex-col gap-3 p-5 text-left">
      <div className="flex items-start justify-between">
        <div>
          <span className="inline-block rounded-md bg-[var(--brand-100)] px-2 py-0.5 text-[10px] font-semibold uppercase text-[var(--brand-700)]">精选</span>
          <h3 className="mt-2 text-[18px] font-semibold">{recipe.name}</h3>
          <p className="text-[12px] text-[var(--fg-soft)]">{recipe.category}</p>
        </div>
        <span className="rounded-lg border border-[var(--border-soft)] px-2 py-1 text-[11px] text-[var(--fg-soft)]">{translateRuntimeKind(app.runtime, lang)}</span>
      </div>
      <p className="text-[13px] leading-relaxed text-[var(--fg-soft)]">{recipe.summary}</p>
      <div className="mt-auto flex items-center gap-2 text-[12px] text-[var(--fg-muted)]">
        <span>端口: {app.ports?.length ? app.ports.join(', ') : '无'}</span>
        <span className="ml-auto font-medium text-[var(--fg-strong)] group-hover:text-[var(--brand-600)]">安装 →</span>
      </div>
    </button>
  );
}

function StandardCard({ app, onClick }: { app: OpenNestRecipeSummary; onClick: () => void }) {
  const { lang, t } = useI18n();
  const recipe = translateRecipeText(app, lang);
  return (
    <button onClick={onClick} className="surface-card group flex flex-col gap-2 p-4 text-left">
      <div className="flex items-start justify-between">
        <h3 className="text-[15px] font-medium">{recipe.name}</h3>
        <span className="rounded-md border border-[var(--border-soft)] px-1.5 py-0.5 text-[10px] text-[var(--fg-soft)]">{translateRuntimeKind(app.runtime, lang)}</span>
      </div>
      <p className="text-[12px] leading-relaxed text-[var(--fg-soft)]">{recipe.summary}</p>
      <div className="mt-auto flex items-center justify-between text-[11px] text-[var(--fg-muted)]">
        <span>{app.category}</span>
        <span className="font-medium text-[var(--fg-strong)] group-hover:text-[var(--brand-600)]">安装 →</span>
      </div>
    </button>
  );
}