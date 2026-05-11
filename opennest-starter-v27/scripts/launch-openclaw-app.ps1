$ErrorActionPreference = "Stop"

$cli = "C:\Users\Lenovo\AppData\Roaming\com.opennest.desktop\apps\openclaw\cli\openclaw.cmd"
$openclawHome = "C:\Users\Lenovo\AppData\Roaming\com.opennest.desktop\apps\openclaw"
$stateDir = Join-Path $openclawHome "state"
$configPath = Join-Path $openclawHome "config\openclaw.json"
$edgePath = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
$host = "127.0.0.1"
$port = 18789

function Test-GatewayReady {
  try {
    $result = Test-NetConnection -ComputerName $host -Port $port -WarningAction SilentlyContinue
    return [bool]$result.TcpTestSucceeded
  } catch {
    return $false
  }
}

function Get-GatewayTokenUrl {
  if (-not (Test-Path $configPath)) {
    throw "OpenClaw config not found: $configPath"
  }

  $config = Get-Content $configPath -Raw | ConvertFrom-Json
  $token = $config.gateway.auth.token
  if ([string]::IsNullOrWhiteSpace($token)) {
    throw "OpenClaw gateway token is missing in $configPath"
  }

  return "http://127.0.0.1:18789/#token=$token"
}

if (-not (Test-Path $cli)) {
  throw "OpenClaw CLI not found: $cli"
}

$env:OPENCLAW_HOME = $openclawHome
$env:OPENCLAW_STATE_DIR = $stateDir
$env:OPENCLAW_CONFIG_PATH = $configPath

if (-not (Test-GatewayReady)) {
  Start-Process -FilePath $cli -ArgumentList "gateway", "start" -WorkingDirectory (Split-Path $cli -Parent) -WindowStyle Hidden
  $deadline = (Get-Date).AddSeconds(30)
  while ((Get-Date) -lt $deadline) {
    if (Test-GatewayReady) {
      break
    }
    Start-Sleep -Milliseconds 500
  }
}

if (-not (Test-GatewayReady)) {
  throw "OpenClaw gateway did not become ready on 127.0.0.1:18789 within 30 seconds."
}

$url = Get-GatewayTokenUrl

if (-not (Test-Path $edgePath)) {
  throw "Microsoft Edge not found: $edgePath"
}

Start-Process -FilePath $edgePath -ArgumentList @("--app=$url", "--window-size=1440,920")
