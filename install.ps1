# ezCMD automated installer for Windows PowerShell
$ErrorActionPreference = "Stop"

$Repo = "ItzEvhyx/ezCMD"
$InstallDir = "$env:USERPROFILE\.local\bin"
$ExePath = "$InstallDir\ezcmd.exe"

Write-Host "Installing ezCMD for Windows..." -ForegroundColor Cyan

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# $PSScriptRoot is empty when piped into iex (irm | iex)
$ScriptDir = $PSScriptRoot
if ($ScriptDir -and (Test-Path "$ScriptDir\target\release\ezcmd.exe")) {
    Copy-Item "$ScriptDir\target\release\ezcmd.exe" $ExePath -Force
} elseif ($ScriptDir -and (Test-Path "$ScriptDir\ezcmd.exe")) {
    Copy-Item "$ScriptDir\ezcmd.exe" $ExePath -Force
} else {
    $AssetTarget = "x86_64-pc-windows-msvc"
    $ZipUrl = ""
    try {
        $ReleaseJson = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "ezcmd-installer" }
        $Asset = $ReleaseJson.assets | Where-Object { $_.name -like "*$AssetTarget*.zip" -and $_.name -notlike "*.sha256" } | Select-Object -First 1
        if ($Asset) {
            $ZipUrl = $Asset.browser_download_url
        }
    } catch {
        # ignore api error/rate limit
    }
    if (-not $ZipUrl) {
        $ZipUrl = "https://github.com/$Repo/releases/download/v2.1.0/ezcmd-v2.1.0-$AssetTarget.zip"
    }

    $TempZip = "$env:TEMP\ezcmd.zip"
    $TempExtract = "$env:TEMP\ezcmd_extract"

    Write-Host "Downloading $ZipUrl..." -ForegroundColor Gray
    Invoke-WebRequest -Uri $ZipUrl -OutFile $TempZip
    Expand-Archive -Path $TempZip -DestinationPath $TempExtract -Force
    Copy-Item "$TempExtract\ezcmd.exe" $ExePath -Force
    Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
    Remove-Item $TempExtract -Recurse -Force -ErrorAction SilentlyContinue
}

# Create configuration marker
$ConfigDir = "$env:APPDATA\ezcmd"
if (-not (Test-Path $ConfigDir)) {
    New-Item -ItemType Directory -Path $ConfigDir -Force | Out-Null
}
Set-Content -Path "$ConfigDir\.installed" -Value "installed" -Force

# Check if ~/.local/bin is in PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "Adding $InstallDir to User PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
}

Write-Host ""
Write-Host "✓ ezcmd was successfully installed to $ExePath!" -ForegroundColor Green
Write-Host ""

try {
    & "$ExePath" --welcome
} catch {
    Write-Host @"
                      /$$$$$$  /$$      /$$ /$$$$$$$ 
                     /$$__  $$| $$$    /$$$| $$__  $$
  /$$$$$$  /$$$$$$$$| $$  \__/| $$$$  /$$$$| $$  \ $$
 /$$__  $$|____ /$$/| $$      | $$ $$/$$ $$| $$  | $$
| $$$$$$$$   /$$$$/ | $$      | $$  $$$| $$| $$  | $$
| $$_____/  /$$__/  | $$    $$| $$\  $ | $$| $$  | $$
|  $$$$$$$ /$$$$$$$$|  $$$$$$/| $$ \/  | $$| $$$$$$$/
 \_______/|________/ \______/ |__/     |__/|_______/ 

  Welcome to ezCMD! Find the right command, in order.
  Never browse Stack Overflow for everyday terminal commands again.

  QUICK START / HOW TO USE:
    ezcmd <query>                   Search for workflows or commands
    ezcmd --help                    Browse categories and tools for your OS
    ezcmd --help -git               Show Git groups and sections
    ezcmd --help -git -worktree     Show one Git section
    ezcmd --uninstall               Uninstall ezcmd from your machine

  WORKFLOWS TO TRY:
    ezcmd initializing repo
    ezcmd undo last commit
    ezcmd how to branch
    ezcmd delete files

  AUTHOR:
    Created by ItzEvhyx
    GitHub: https://github.com/ItzEvhyx
    Repo:   https://github.com/ItzEvhyx/ezCMD
"@
}
