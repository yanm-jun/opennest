# Changelog

## v0.1.0 — Platformization Release (2026-05-11)

### Architecture
- **App Manifest System**: 8 universal types (`OpenNestAppManifest`, `OpenNestAppStatus`, `OpenNestInstallType`, `OpenNestAppSourceType`, `OpenNestAppPermissions`, `OpenNestSystemRequirements`, `OpenNestInstallRecipe`, `OpenNestRuntimeConfig`)
- **App Registry**: `src/data/apps.ts` as single source of truth (4 apps: openclaw, open-webui, flowise, dify)
- **App Registry Helper**: `src/lib/app-registry.ts` (`listApps`, `getAppOrThrow`)

### De-hardcoding
- OpenClaw demoted from platform core to `apps[0]`
- All install/start/stop/open/logs/status driven by `appId`
- `isOpenClaw` → `hasSecrets` (provider panel), `runtime === "native-cli"` (port label), dynamic gates (ValidationBoard)
- i18n: 8 dead `openclaw_*` keys removed, `gate_*` parametric keys added (en/zh)
- Types: `OpenClawProvider` → `AppProvider`, `OpenClawSetupInput` → `AppSecretsSetupInput`
- `recipeProfiles.ts`: `openclaw` variable → `isNativeCli`

### Rust Backend
- `native_cli.rs`: 7 functions renamed (`install_openclaw` → `install_app(app_id)`, etc.)
- `runtime_router.rs`: dispatch by `runtime` instead of `recipe.id == "openclaw"`
- `rollback_adapter.rs`: `stop_openclaw` → `stop_app`
- `commands.rs`: 31 total Tauri commands (22 core + 9 stub)
- `install_plan.rs`: warnings + binary placeholder genericized

### UI Polish
- Secret status indicator (saved ✓)
- Inline uninstall confirmation (replaces `window.confirm`)
- Animated progress bar with state color dots
- App search + category filter in AppCenter

### Infra
- `scripts/copy-into-opennest.ps1`: one-click Tauri integration
- `.github/workflows/ci.yml`: frontend build + Rust cargo check
- 183 files, ~33,000 lines in repository