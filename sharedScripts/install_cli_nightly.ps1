[cmdletbinding()]
param(
   [string]$RunId,
   [string]$Branch,
   [string]$Repo,
   [string]$InstallPath
)

$ErrorActionPreference="Stop"

if ($null -eq (Get-Command "gh" -ErrorAction SilentlyContinue)) { 
    throw "Please install the GitHub CLI: https://cli.github.com/"
}

$platform = if ($IsWindows) { "windows" } elseif ($IsLinux) { "linux" } elseif ($IsMacOS) { "macos" } else { throw "Unsupported OS" }

# Fetch
if (!$InstallPath) {
  # Default install paths by platform
  if ($IsWindows) {
    $InstallPath = [System.IO.Path]::combine($env:LOCALAPPDATA, "dsc")
  } else {
    $InstallPath = [System.IO.Path]::combine($HOME, ".dsc", "bin")
  }
}
if (!$Repo) {
  $Repo = "PowerShell/DSC"
}
if (!$Branch) {
  $Branch = "main"
}
if (!$RunId) {
  $RunId = & gh run list -R $Repo --branch $Branch --workflow rust --status success -L 1 --json databaseId -q ".[0].databaseId"; if(!$?) { throw }
  if (!$RunId) {
    throw "Failed to find a successful build to install from"
  }
}

$tmpDir = [System.IO.Path]::combine([System.IO.Path]::GetTempPath(), [System.IO.Path]::GetRandomFileName())
& gh run download -R $Repo $RunId -n "$platform-bin" --dir $tmpDir; if(!$?) { throw }

$tar = Get-ChildItem -Path $tmpDir | Select-Object -First 1
if (!$tar) {
  throw "Failed to find downloaded artifact"
}

if (-not (Get-Command "tar" -ErrorAction SilentlyContinue)) {
    throw "Please install 'tar' to extract the downloaded artifact."
}

tar -xf $tar.FullName -C $tmpDir; if(!$?) { throw }

$installationFiles = Join-Path $tmpDir 'bin' 'debug'
New-Item -ItemType Directory -Force -Path $InstallPath | Out-Null
Move-Item -Path "$installationFiles/*" -Destination $InstallPath -Force -ErrorAction Ignore

$dscExe = if ($IsWindows) { Join-Path $InstallPath "dsc.exe" } else { Join-Path $InstallPath "dsc" }
$versionStdout = & $dscExe --version; if(!$?) { throw }
$version = $versionStdout -replace 'dsc ', ''
Write-Host "Installed DSC CLI $version from https://github.com/$Repo/actions/runs/$RunId to $InstallPath"
Write-Host "Make sure to add $InstallPath to your PATH environment variable to use the 'dsc' command."

# Cleanup
Remove-Item $tmpDir -Recurse