param(
    [switch]$Install
)

$ErrorActionPreference = "Stop"

function Test-Command {
    param([Parameter(Mandatory = $true)][string]$Name)
    return ($null -ne (Get-Command $Name -ErrorAction SilentlyContinue))
}

function Get-CommandVersion {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [string[]]$Arguments = @("--version")
    )

    if (-not (Test-Command -Name $Name)) {
        return ""
    }

    try {
        return ((& $Name @Arguments 2>$null) | Select-Object -First 1)
    }
    catch {
        return ""
    }
}

function Write-Check {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][bool]$Ok,
        [string]$Detail = "",
        [string]$Fix = ""
    )

    $mark = if ($Ok) { "[OK]" } else { "[MISS]" }
    $line = if ([string]::IsNullOrWhiteSpace($Detail)) { "$mark $Name" } else { "$mark $Name - $Detail" }
    Write-Host $line
    if (-not $Ok -and -not [string]::IsNullOrWhiteSpace($Fix)) {
        Write-Host "      Fix: $Fix"
    }
}

function Invoke-WingetInstall {
    param(
        [Parameter(Mandatory = $true)][string]$PackageId,
        [string[]]$ExtraArguments = @()
    )

    if (-not (Test-Command -Name "winget")) {
        throw "winget was not found. Install App Installer from Microsoft Store, then rerun this script."
    }

    $arguments = @(
        "install",
        "--id", $PackageId,
        "-e",
        "--source", "winget",
        "--accept-source-agreements",
        "--accept-package-agreements"
    ) + $ExtraArguments

    Write-Host "Installing $PackageId ..."
    & winget @arguments
    if ($LASTEXITCODE -ne 0) {
        throw "winget install failed for $PackageId with exit code $LASTEXITCODE."
    }
}

function Import-FnmEnvironment {
    if (Test-Command -Name "fnm") {
        fnm env --use-on-cd --shell powershell | Out-String | Invoke-Expression
    }
}

function Test-MsvcBuildTools {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path -LiteralPath $vswhere -PathType Leaf)) {
        return ""
    }

    $installationPath = & $vswhere -products * -latest -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ([string]::IsNullOrWhiteSpace($installationPath)) {
        return ""
    }

    return [string]$installationPath
}

function Test-WebView2Runtime {
    $candidateRoots = @(
        (Join-Path ${env:ProgramFiles(x86)} "Microsoft\EdgeWebView\Application"),
        (Join-Path $env:ProgramFiles "Microsoft\EdgeWebView\Application")
    )

    foreach ($root in $candidateRoots) {
        if (Test-Path -LiteralPath $root -PathType Container) {
            $runtime = Get-ChildItem -LiteralPath $root -Recurse -Filter "msedgewebview2.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
            if ($null -ne $runtime) {
                return $runtime.FullName
            }
        }
    }

    $registryPaths = @(
        "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9C2BB00}",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9C2BB00}",
        "HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9C2BB00}"
    )

    foreach ($path in $registryPaths) {
        if (Test-Path -LiteralPath $path) {
            $value = (Get-ItemProperty -LiteralPath $path -ErrorAction SilentlyContinue).pv
            if (-not [string]::IsNullOrWhiteSpace($value)) {
                return "WebView2 Runtime $value"
            }
        }
    }

    return ""
}

if ($env:OS -ne "Windows_NT") {
    throw "This script is for Windows. Use scripts/check-env.sh on macOS."
}

Write-Host "Checking Tauri development environment for Windows..."
Write-Host ""

$fnmInstalled = Test-Command -Name "fnm"
if (-not $fnmInstalled -and $Install) {
    Invoke-WingetInstall -PackageId "Schniz.fnm"
    Write-Host "fnm was installed. If this shell still cannot find fnm, reopen PowerShell and rerun this script."
}

Import-FnmEnvironment

$nodeInstalled = Test-Command -Name "node"
$npmInstalled = Test-Command -Name "npm"

if ((-not $nodeInstalled -or -not $npmInstalled) -and (Test-Command -Name "fnm") -and $Install) {
    Write-Host "Installing Node.js LTS with fnm..."
    fnm install --lts
    fnm default lts-latest
    Import-FnmEnvironment
    $nodeInstalled = Test-Command -Name "node"
    $npmInstalled = Test-Command -Name "npm"
}

$rustupInstalled = Test-Command -Name "rustup"
$cargoInstalled = Test-Command -Name "cargo"
$rustcInstalled = Test-Command -Name "rustc"

if ((-not $cargoInstalled -or -not $rustcInstalled) -and $Install) {
    Invoke-WingetInstall -PackageId "Rustlang.Rustup"
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if ((Test-Path -LiteralPath $cargoBin -PathType Container) -and $env:Path -notlike "*$cargoBin*") {
        $env:Path += ";$cargoBin"
    }
    $rustupInstalled = Test-Command -Name "rustup"
    $cargoInstalled = Test-Command -Name "cargo"
    $rustcInstalled = Test-Command -Name "rustc"
}

$msvcPath = Test-MsvcBuildTools
if ([string]::IsNullOrWhiteSpace($msvcPath) -and $Install) {
    Invoke-WingetInstall -PackageId "Microsoft.VisualStudio.2022.BuildTools" -ExtraArguments @(
        "--override",
        "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
    )
    $msvcPath = Test-MsvcBuildTools
}

$webView2 = Test-WebView2Runtime
if ([string]::IsNullOrWhiteSpace($webView2) -and $Install) {
    Invoke-WingetInstall -PackageId "Microsoft.EdgeWebView2Runtime"
    $webView2 = Test-WebView2Runtime
}

Write-Host ""
Write-Check -Name "fnm" -Ok (Test-Command -Name "fnm") -Detail (Get-CommandVersion -Name "fnm") -Fix "winget install Schniz.fnm"
Write-Check -Name "node" -Ok (Test-Command -Name "node") -Detail (Get-CommandVersion -Name "node") -Fix "fnm install --lts; fnm default lts-latest"
Write-Check -Name "npm" -Ok (Test-Command -Name "npm") -Detail (Get-CommandVersion -Name "npm") -Fix "Install Node.js through fnm."
Write-Check -Name "rustup" -Ok (Test-Command -Name "rustup") -Detail (Get-CommandVersion -Name "rustup") -Fix "winget install Rustlang.Rustup"
Write-Check -Name "rustc" -Ok (Test-Command -Name "rustc") -Detail (Get-CommandVersion -Name "rustc") -Fix "Install Rust through rustup."
Write-Check -Name "cargo" -Ok (Test-Command -Name "cargo") -Detail (Get-CommandVersion -Name "cargo") -Fix "Install Rust through rustup."
Write-Check -Name "MSVC Build Tools" -Ok (-not [string]::IsNullOrWhiteSpace($msvcPath)) -Detail $msvcPath -Fix "winget install Microsoft.VisualStudio.2022.BuildTools, include Desktop development with C++."
Write-Check -Name "WebView2 Runtime" -Ok (-not [string]::IsNullOrWhiteSpace($webView2)) -Detail $webView2 -Fix "winget install Microsoft.EdgeWebView2Runtime"

Write-Host ""
if ((Test-Command -Name "node") -and (Test-Command -Name "npm") -and (Test-Command -Name "cargo") -and (Test-Command -Name "rustc") -and (-not [string]::IsNullOrWhiteSpace($msvcPath)) -and (-not [string]::IsNullOrWhiteSpace($webView2))) {
    Write-Host "Environment looks ready. Try: npm run tauri:dev"
}
else {
    Write-Host "Environment is not ready yet."
    if (-not $Install) {
        Write-Host "Run with installation enabled:"
        Write-Host "  powershell -NoProfile -ExecutionPolicy Bypass -File scripts\check-env.ps1 -Install"
    }
    else {
        Write-Host "Some installers may require a reopened terminal or manual component selection."
    }
}
