<#
.SYNOPSIS
    Copies the drop-in recipe runtime source and v27 frontend into an existing OpenNest Tauri project.

.PARAMETER TauriRoot
    Path to the OpenNest Tauri project root (the folder that contains src-tauri/).
#>
param(
    [Parameter(Mandatory)]
    [string]$TauriRoot
)

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $scriptDir "..")

Write-Host "OpenNest → Tauri integration" -ForegroundColor Cyan
Write-Host "  Repository root : $repoRoot"
Write-Host "  Tauri target     : $TauriRoot"
Write-Host ""

# 1. Copy Rust recipe_runtime module
$rustSrc = Join-Path $repoRoot "drop-in\src-tauri\src\recipe_runtime"
$rustDest = Join-Path $TauriRoot "src-tauri\src\recipe_runtime"

if (-not (Test-Path $rustSrc)) {
    Write-Error "Rust source not found: $rustSrc"
    exit 1
}

Write-Host "[1/5] Copying Rust recipe_runtime/ ..." -ForegroundColor Yellow
Copy-Item -Recurse -Force $rustSrc $rustDest
Write-Host "  Done. $((Get-ChildItem $rustDest -Recurse -File).Count) files copied."

# 2. Copy registry and recipes
Write-Host "[2/5] Copying registry/ and recipes/ ..." -ForegroundColor Yellow
$regSrc = Join-Path $repoRoot "registry"
$recSrc = Join-Path $repoRoot "recipes"
$regDest = Join-Path $TauriRoot "registry"
$recDest = Join-Path $TauriRoot "recipes"

if (Test-Path $regDest) { Remove-Item -Recurse -Force $regDest }
if (Test-Path $recDest) { Remove-Item -Recurse -Force $recDest }
Copy-Item -Recurse $regSrc $regDest
Copy-Item -Recurse $recSrc $recDest
Write-Host "  Done."

# 3. Build v27 frontend
$frontendSrc = Join-Path $repoRoot "opennest-starter-v27"
Write-Host "[3/5] Building v27 frontend ..." -ForegroundColor Yellow
Push-Location $frontendSrc
npm install --silent 2>$null
npm run build
Pop-Location

# 4. Copy frontend dist into Tauri
$distSrc = Join-Path $frontendSrc "dist"
$distDest = Join-Path $TauriRoot "dist"
Write-Host "[4/5] Copying frontend dist/ ..." -ForegroundColor Yellow
if (Test-Path $distDest) { Remove-Item -Recurse -Force $distDest }
Copy-Item -Recurse $distSrc $distDest
Write-Host "  Done."

# 5. Print manual steps
Write-Host "[5/5] Manual steps" -ForegroundColor Yellow
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "  A) src-tauri/src/main.rs" -ForegroundColor White
Write-Host "     Add near the top:"
Write-Host "       mod recipe_runtime;"
Write-Host ""
Write-Host "     Inside generate_handler![...] add every command"
Write-Host "     listed in snippets/register-recipe-runtime-main.rs"
Write-Host ""
Write-Host "  B) src-tauri/Cargo.toml" -ForegroundColor White
Write-Host "     Add dependencies listed in"
Write-Host "     snippets/cargo-dependencies.toml"
Write-Host ""
Write-Host "  C) src-tauri/tauri.conf.json" -ForegroundColor White
Write-Host "     Set frontendDist to dist/"
Write-Host ""
Write-Host "  D) Run cargo build and cargo tauri dev" -ForegroundColor White
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

Write-Host "`n✓ Integration files copied. Follow manual steps A-D above." -ForegroundColor Green