# Bump installer — Windows (PowerShell)
#
# Installs to %LOCALAPPDATA%\Programs\bump\bump.exe and adds that folder to your user PATH unless -SkipPath.
#
# Usage:
#   irm https://raw.githubusercontent.com/launchfirestorm/bump/main/install/get_bump.ps1 | iex
#
# From a clone:
#   .\install\get_bump.ps1
#
# Requires: Windows PowerShell 5.1+ or PowerShell 7+.

param(
  [string]$InstallDir = '',
  [switch]$SkipPath
)

$ErrorActionPreference = 'Stop'

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$script:GithubHeaders = @{
  'User-Agent' = 'bump-windows-installer'
  'Accept'     = 'application/vnd.github+json'
}

function Write-Log {
  param(
    [ValidateSet('Info', 'Ok', 'Warn', 'Err')]
    [string]$Level,
    [string]$Message
  )
  switch ($Level) {
    'Info' { Write-Host "[INFO] $Message" -ForegroundColor White }
    'Ok' { Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
    'Warn' { Write-Host "[WARNING] $Message" -ForegroundColor Yellow }
    'Err' { Write-Host "[ERROR] $Message" -ForegroundColor Red; exit 1 }
  }
}

$Repo = 'launchfirestorm/bump'
$BinaryName = 'bump.exe'

function Get-LatestReleaseTag {
  try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers $script:GithubHeaders
    if (-not $release.tag_name) { return $null }
    return $release.tag_name.Trim()
  } catch {
    return $null
  }
}

function Get-TargetArch {
  switch ($env:PROCESSOR_ARCHITECTURE) {
    'AMD64' { return 'amd64' }
    'ARM64' { return 'arm64' }
    default { return $null }
  }
}

function Add-UserPathEntry {
  param([string]$Directory)
  $d = $Directory.TrimEnd('\')
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  if (-not $userPath) {
    [Environment]::SetEnvironmentVariable('Path', $d, 'User')
    return $true
  }
  $escaped = [regex]::Escape($d)
  if ($userPath -match "(?i)(^|;)$escaped(;|$)") { return $false }
  [Environment]::SetEnvironmentVariable('Path', "$userPath;$d", 'User')
  return $true
}

Write-Host ''
Write-Host 'Bump installer (Windows)' -ForegroundColor Cyan
Write-Host ''

$arch = Get-TargetArch
if (-not $arch) { Write-Log Err "Unsupported CPU architecture (need amd64 or arm64)." }

$assetName = "bump-windows-$arch.exe"
Write-Log Info "Platform: Windows ($arch)"

if (-not $InstallDir) {
  $InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\bump'
}
$InstallDir = [System.IO.Path]::GetFullPath($InstallDir)
$targetPath = Join-Path $InstallDir $BinaryName

$tag = Get-LatestReleaseTag
if (-not $tag) { Write-Log Err 'Could not resolve latest release from GitHub.' }
Write-Log Info "Latest release: $tag"

$downloadUrl = "https://github.com/$Repo/releases/download/$tag/$assetName"
Write-Log Info "Downloading: $downloadUrl"

$tempFile = Join-Path ([System.IO.Path]::GetTempPath()) ('bump-install-' + [Guid]::NewGuid().ToString('N') + '.exe')
try {
  Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
} catch {
  Remove-Item -LiteralPath $tempFile -ErrorAction SilentlyContinue
  Write-Log Err "Download failed: $($_.Exception.Message)"
}

if (-not (Test-Path -LiteralPath $tempFile) -or ((Get-Item -LiteralPath $tempFile).Length -eq 0)) {
  Remove-Item -LiteralPath $tempFile -ErrorAction SilentlyContinue
  Write-Log Err 'Downloaded file is missing or empty.'
}

if (-not (Test-Path -LiteralPath $InstallDir)) {
  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

try {
  if (Test-Path -LiteralPath $targetPath) {
    Remove-Item -LiteralPath $targetPath -Force
  }
  Move-Item -LiteralPath $tempFile -Destination $targetPath -Force
} catch {
  Remove-Item -LiteralPath $tempFile -ErrorAction SilentlyContinue
  Write-Log Err "Failed to install to ${targetPath}: $($_.Exception.Message)"
}

Write-Log Ok "Installed: $targetPath"

if (-not $SkipPath) {
  if (Add-UserPathEntry -Directory $InstallDir) {
    Write-Log Ok "Added to user PATH: $InstallDir"
  } else {
    Write-Log Info "PATH already contains: $InstallDir"
  }
  $env:Path = $env:Path + ';' + $InstallDir
}

try {
  $ver = & $targetPath --version 2>&1
  Write-Log Ok "Verification: $ver"
} catch {
  Write-Log Warn 'Could not run bump --version. Open a new terminal and try again.'
}

Write-Host ''
Write-Log Info 'Done. If bump was not found, close and reopen your terminal.' -ForegroundColor Green
Write-Host ''
