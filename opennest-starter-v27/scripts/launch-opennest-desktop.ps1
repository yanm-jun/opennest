$ErrorActionPreference = "Stop"

$projectRoot = "C:\Users\Lenovo\Desktop\work_v24\opennest-starter-v27"
$frontendUrl = "http://127.0.0.1:1420"
$frontendHost = "127.0.0.1"
$frontendPort = 1420
$desktopExe = Join-Path $projectRoot "src-tauri\target\debug\opennest-desktop-starter.exe"

function Test-FrontendReady {
  try {
    $result = Test-NetConnection -ComputerName $frontendHost -Port $frontendPort -WarningAction SilentlyContinue
    return [bool]$result.TcpTestSucceeded
  } catch {
    return $false
  }
}

if (-not (Test-FrontendReady)) {
  Start-Process -FilePath "npm.cmd" -ArgumentList "run", "dev" -WorkingDirectory $projectRoot -WindowStyle Hidden
  $deadline = (Get-Date).AddSeconds(30)
  while ((Get-Date) -lt $deadline) {
    if (Test-FrontendReady) {
      break
    }
    Start-Sleep -Milliseconds 500
  }
}

if (-not (Test-FrontendReady)) {
  throw "OpenNest frontend did not become ready at $frontendUrl within 30 seconds."
}

$existing = Get-Process opennest-desktop-starter -ErrorAction SilentlyContinue
if ($existing) {
  $existing | Stop-Process -Force
  Start-Sleep -Seconds 1
}

Start-Process -FilePath $desktopExe -WorkingDirectory (Split-Path $desktopExe -Parent)
