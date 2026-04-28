param(
    [switch]$CheckOnly
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $projectRoot

Write-Host "Starting Terminal Pane Launcher Tauri on Windows..."
Write-Host ""

if (Get-Command fnm -ErrorAction SilentlyContinue) {
    fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
}

$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path -LiteralPath $cargoBin -PathType Container) -and $env:Path -notlike "*$cargoBin*") {
    $env:Path += ";$cargoBin"
}

if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
    throw "npm was not found. Run the Windows environment installer first, then reopen this launcher."
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found. Run the Windows environment installer first, then reopen this launcher."
}

if ($CheckOnly) {
    Write-Host ("npm: {0}" -f (Get-Command npm).Source)
    Write-Host ("cargo: {0}" -f (Get-Command cargo).Source)
    Write-Host "Start script preflight passed."
    return
}

if (-not (Test-Path -LiteralPath (Join-Path $projectRoot "node_modules") -PathType Container)) {
    Write-Host "Installing npm dependencies..."
    npm install
    if ($LASTEXITCODE -ne 0) {
        throw "npm install failed with exit code $LASTEXITCODE."
    }
}

npm run tauri:dev
if ($LASTEXITCODE -ne 0) {
    throw "Tauri launch failed with exit code $LASTEXITCODE. Run npm run env to check the environment."
}
