# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

param(
    [switch]$Release,
    [ValidateSet('none','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc')]
    $architecture = 'none',
    [switch]$Clippy,
    [switch]$Test
)

## Test if Rust is installed
if (!(Get-Command 'cargo' -ErrorAction Ignore)) {
    Write-Verbose -Verbose "Rust not found, installing..."
    if (!$IsWindows) {
        curl https://sh.rustup.rs -sSf | sh
    }
    else {
        Invoke-WebRequest 'https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe' -OutFile 'temp:/rustup-init.exe'
        & 'temp:/rustup-init.exe'
        Remove-Item temp:/rustup-init.exe -ErrorAction Ignore
    }
}

## Create the output folder
$configuration = $Release ? 'release' : 'debug'
$target = Join-Path $PSScriptRoot 'bin' $configuration
if (Test-Path $target) {
    Remove-Item $target -Recurse -ErrorAction Stop
}
New-Item -ItemType Directory $target > $null

$flags = @($Release ? '-r' : $null)
if ($architecture -ne 'none') {
    $flags += '--target'
    $flags += $architecture
    $path = ".\target\$architecture\$configuration"
}
else {
    $path = ".\target\$configuration"
}

$windows_projects = @("pal", "ntreg", "ntstatuserror", "ntuserinfo", "registry")
$projects = @("dsc_lib", "dsc", "osinfo", "test_group_resource", "y2j")
$pedantic_clean_projcets = @("dsc_lib", "dsc", "osinfo", "y2j", "pal", "ntstatuserror", "ntuserinfo", "test_group_resource")

if ($IsWindows) {
    $projects += $windows_projects
}

Copy-Item "$PSScriptRoot/powershellgroup/*" $target -Force -ErrorAction Ignore

$failed = $false
foreach ($project in $projects) {
    ## Build format_json
    Write-Host -ForegroundColor Cyan "Building $project ..."
    try {
        Push-Location "$PSScriptRoot/$project" -ErrorAction Stop
        if ($Clippy) {
            if ($pedantic_clean_projcets -contains $project) {
                Write-Verbose -Verbose "Running clippy with pedantic for $project"
                cargo clippy @flags --% -- -Dwarnings -Dclippy::pedantic
            }
            else {
                Write-Verbose -Verbose "Running clippy for $project"
                cargo clippy @flags -- -Dwarnings
            }
        }
        else {
            cargo build @flags
        }

        if ($LASTEXITCODE -ne 0) {
            $failed = $true
        }

        if ($IsWindows) {
            Copy-Item "$path/$project.exe" $target -ErrorAction Ignore
        }
        else {
            Copy-Item "$path/$project" $target -ErrorAction Ignore
        }

        Copy-Item "*.resource.json" $target -Force -ErrorAction Ignore
        Copy-Item "*.command.json" $target -Force -ErrorAction Ignore

    } finally {
        Pop-Location
    }
}

if ($failed) {
    Write-Host -ForegroundColor Red "Build failed"
    exit 1
}

$relative = Resolve-Path $target -Relative
Write-Host -ForegroundColor Green "`nEXE's are copied to $target ($relative)"

$paths = $env:PATH.Split([System.IO.Path]::PathSeparator)
$found = $false
foreach ($path in $paths) {
    if ($path -eq $target) {
        $found = $true
        break
    }
}

# remove the other target in case switching between them
if ($Release) {
    $oldTarget = $target.Replace('\release', '\debug')
}
else {
    $oldTarget = $target.Replace('\debug', '\release')
}
$env:PATH = $env:PATH.Replace(';' + $oldTarget, '')

if (!$found) {
    Write-Host -ForegroundCOlor Yellow "Adding $target to `$env:PATH"
    $env:PATH += [System.IO.Path]::PathSeparator + $target
}

if ($Test) {
    $failed = $false
    
    "Installing module PSDesiredStateConfiguration 2.0.7"
    Set-PSRepository -Name 'PSGallery' -InstallationPolicy Trusted
    Install-module PSDesiredStateConfiguration -RequiredVersion 2.0.7
    "Installing module Pester"
    Install-module Pester -WarningAction Ignore

    "For debug - env:PATH is:"
    $env:PATH

    # On Windows remove duplicated WinPS resources that break PSDesiredStateConfiguration module
    if ($IsWindows) {
        $a = $env:PSModulePath -split ";" | ? { $_ -notmatch 'WindowsPowerShell' }
        $env:PSModulePath = $a -join ';'
    }

    foreach ($project in $projects) {
        ## Build format_json
        Write-Host -ForegroundColor Cyan "Testing $project ..."
        try {
            Push-Location "$PSScriptRoot/$project"
            cargo test

            if ($LASTEXITCODE -ne 0) {
                $failed = $true
            }
        } finally {
            Pop-Location
        }
    }

    if ($failed) {
        throw "Test failed"
    }

    Invoke-Pester -ErrorAction Stop
}

$env:RUST_BACKTRACE=1
