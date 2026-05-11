import { useMemo, useState } from "react";
import { RecipeAppCenter } from "./features/recipes/RecipeAppCenter";
import { RecipeDetailPage } from "./features/recipes/RecipeDetailPage";
import { MyLibraryRecipes } from "./features/recipes/MyLibraryRecipes";
import { ValidationBoard } from "./features/recipes/ValidationBoard";
import { ErrorCenter } from "./features/recipes/ErrorCenter";
import { useI18n } from "./i18n";

type View = "apps" | "library" | "validation" | "errors" | "settings";

// v27.1
export default function App() { // BUILD_MARKER_2026
  const { lang, setLang, t } = useI18n();
  const [view, setView] = useState<View>("apps");
  const [selectedAppId, setSelectedAppId] = useState<string | null>(null);

  const navItems: Array<{ id: View; label: string }> = [
    { id: "apps", label: t("apps") },
    { id: "library", label: t("my_library") },
    { id: "validation", label: t("health") },
    { id: "errors", label: t("errors") },
    { id: "settings", label: t("settings") },
  ];

  const showDetail = selectedAppId != null;

  return (
    <div className="min-h-screen bg-[var(--app-bg)] text-[var(--fg-strong)]">
      <header className="sticky top-0 z-20 border-b border-[var(--border-subtle)] bg-[var(--app-bg)]/95 backdrop-blur">
        <div className="mx-auto flex h-14 max-w-[1200px] items-center justify-between px-6">
          <div className="flex items-center gap-6">
            <span className="text-[16px] font-semibold tracking-tight">OpenNest 桌面</span>
            <nav className="flex gap-1">
              {navItems.map((item) => (
                <button
                  key={item.id}
                  onClick={() => { setSelectedAppId(null); setView(item.id); }}
                  className={"rounded-lg px-3 py-1.5 text-[13px] font-medium transition-colors " +
                    (view === item.id && !showDetail
                      ? "bg-[var(--bg-strong)] text-[var(--fg-strong)]"
                      : "text-[var(--fg-soft)] hover:text-[var(--fg-strong)]")}
                >
                  {item.label}
                </button>
              ))}
            </nav>
          </div>
          <div className="flex items-center gap-2">
            <select
              value={lang}
              onChange={(e) => setLang(e.target.value as "en" | "zh")}
              className="rounded-lg border border-[var(--border-soft)] bg-transparent px-2 py-1 text-[12px] text-[var(--fg-soft)]"
            >
              <option value="en">EN</option>
              <option value="zh">CN</option>
            </select>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-[1200px] px-6 py-6">
        {showDetail ? (
          <RecipeDetailPage appId={selectedAppId} onBack={() => setSelectedAppId(null)} />
        ) : view === "apps" ? (
          <RecipeAppCenter onOpenApp={setSelectedAppId} />
        ) : view === "library" ? (
          <MyLibraryRecipes
            onOpenDetails={setSelectedAppId}
            onOpenErrorCenter={() => { setSelectedAppId(null); setView("errors"); }}
          />
        ) : view === "validation" ? (
          <ValidationBoard onOpenDetails={setSelectedAppId} />
        ) : view === "errors" ? (
          <ErrorCenter onOpenDetails={setSelectedAppId} />
        ) : (
          <设置Panel onOpenCenter={() => setView("apps")} />
        )}
      </main>
    </div>
  );
}

function 设置Panel({ onOpenCenter }: { onOpenCenter: () => void }) {
  const { t } = useI18n();
  return (
    <section className="grid gap-4">
      <div className="surface-card p-6">
        <h2 className="text-[24px] font-semibold">{t("runtime_settings")}</h2>
        <p className="mt-2 text-[14px] text-[var(--fg-soft)]">{t("runtime_settings_desc")}</p>
      </div>

      <div className="grid gap-4 lg:grid-cols-2">
        <div className="surface-card flex flex-col gap-4 p-6">
          <h3 className="text-[16px] font-semibold">{t("prerequisite_checklist")}</h3>
          <p className="text-[12px] text-[var(--fg-soft)]">{t("prerequisite_desc")}</p>
          <ul className="flex flex-col gap-2 text-[14px] text-[var(--fg-soft)]">
            {["Rust 工具链", "Node.js 22.16+ or 24", "Tauri CLI", "Docker Desktop", "Git"].map((item) => (
              <li key={item} className="flex items-center gap-2">
                <span className="h-1.5 w-1.5 rounded-full bg-[var(--brand-600)]" />
                {item}
              </li>
            ))}
          </ul>
        </div>

        <div className="surface-card flex flex-col gap-4 p-6">
          <h3 className="text-[16px] font-semibold">关于</h3>
          <p className="text-[12px] text-[var(--fg-soft)]">
            OpenNest 桌面启动器 v0.27。一键在本地运行开源 AI 应用。
          </p>
          <button onClick={onOpenCenter} className="button-primary mt-auto h-10">
            {t("open_app_center")}
          </button>
        </div>
      </div>
    </section>
  );
}