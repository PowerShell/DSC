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
        Write-Verbose -Verbose "Use the default settings to ensure build works"
        & 'temp:/rustup-init.exe'
        Remove-Item temp:/rustup-init.exe -ErrorAction Ignore
    }
}

$BuildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"

function Find-LibPath {
    Write-Verbose -Verbose "Finding lib path"
    try {
        Push-Location "${env:ProgramFiles(x86)}\Windows Kits\10\lib"
        $arch = $env:PROCESSOR_ARCHITECTURE
        if ($arch -eq 'AMD64') {
            $arch = 'x64'
        }
        Set-Location "$(Get-ChildItem -Directory | Sort-Object name -Descending | Select-Object -First 1)\um\$arch" -ErrorAction Stop
        $libPath = (Get-Location).Path
        Write-Verbose -Verbose "Using $libPath"
        $libPath
    }
    finally {
        Pop-Location
    }
}

function Find-LinkExe {
    try {
        # unfortunately, vs buildtools installer doesn't simply add the exes to PATH so we have to search for them
        Write-Verbose -Verbose "Finding link.exe"
        Push-Location $BuildToolsPath
        Set-Location "$(Get-ChildItem -Directory | Sort-Object name -Descending | Select-Object -First 1)\bin\Host$($env:PROCESSOR_ARCHITECTURE)\x64" -ErrorAction Stop
        $linkexe = (Get-Location).Path
        Write-Verbose -Verbose "Using $linkexe"
        $linkexe
    }
    finally {
        Pop-Location
    }
}

if ($IsWindows -and !(Get-Command 'link.exe' -ErrorAction Ignore)) {
    if (!(Test-Path $BuildToolsPath)) {
        Write-Verbose -Verbose "link.exe not found, installing C++ build tools"
        Invoke-WebRequest 'https://aka.ms/vs/17/release/vs_BuildTools.exe' -OutFile 'temp:/vs_buildtools.exe'
        $args = @('--passive','--add','Microsoft.VisualStudio.Workload.VCTools','--includerecommended')
        if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
            $args += '--add','Microsoft.VisualStudio.Component.VC.Tools.ARM64'
        }
        Start-Process -FilePath 'temp:/vs_buildtools.exe' -ArgumentList $args -Wait
        Remove-Item temp:/vs_installer.exe -ErrorAction Ignore
        Write-Verbose -Verbose "Updating env vars"
        $machineEnv = [environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::Machine).Split(';')
        $userEnv = [environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User).Split(';')
        $pathEnv = ($env:PATH).Split(';')
        foreach ($env in $machineEnv) {
            if ($pathEnv -notcontains $env) {
                $pathEnv += $env
            }
        }
        foreach ($env in $userEnv) {
            if ($pathEnv -notcontains $env) {
                $pathEnv += $env
            }
        }
        $env:PATH = $pathEnv -join ';'
    }

    $linkexe = Find-LinkExe
    $env:PATH += ";$linkexe"
}

if ($IsWindows -and $null -eq $env:MS_SDK_BASE_DOS) {
    $env:MS_SDK_BASE_DOS = Find-LibPath
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
