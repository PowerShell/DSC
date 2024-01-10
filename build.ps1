# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

param(
    [switch]$Release,
    [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
    $architecture = 'current',
    [switch]$Clippy,
    [switch]$Test,
    [switch]$GetPackageVersion,
    [switch]$SkipLinkCheck
)

if ($GetPackageVersion) {
    $match = Select-String -Path $PSScriptRoot/dsc/Cargo.toml -Pattern '^version\s*=\s*"(?<ver>.*?)"$'
    if ($null -eq $match) {
        throw 'Unable to find version in Cargo.toml'
    }

    return $match.Matches.Groups[1].Value
}

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

rustup default stable
$BuildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"

function Find-LinkExe {
    try {
        # this helper may not be needed anymore, but keeping in case the install doesn't work for everyone
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

if (!$SkipLinkCheck -and $IsWindows -and !(Get-Command 'link.exe' -ErrorAction Ignore)) {
    if (!(Test-Path $BuildToolsPath)) {
        Write-Verbose -Verbose "link.exe not found, installing C++ build tools"
        Invoke-WebRequest 'https://aka.ms/vs/17/release/vs_BuildTools.exe' -OutFile 'temp:/vs_buildtools.exe'
        $arg = @('--passive','--add','Microsoft.VisualStudio.Workload.VCTools','--includerecommended')
        if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
            $arg += '--add','Microsoft.VisualStudio.Component.VC.Tools.ARM64'
        }
        Start-Process -FilePath 'temp:/vs_buildtools.exe' -ArgumentList $arg -Wait
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

    #$linkexe = Find-LinkExe
    #$env:PATH += ";$linkexe"
}

## Create the output folder
$configuration = $Release ? 'release' : 'debug'
$flags = @($Release ? '-r' : $null)
if ($architecture -eq 'current') {
    $path = ".\target\$configuration"
    $target = Join-Path $PSScriptRoot 'bin' $configuration
}
else {
    rustup target add $architecture
    $flags += '--target'
    $flags += $architecture
    $path = ".\target\$architecture\$configuration"
    $target = Join-Path $PSScriptRoot 'bin' $architecture $configuration
}

if (Test-Path $target) {
    Remove-Item $target -Recurse -ErrorAction Stop
}
New-Item -ItemType Directory $target > $null

# make sure dependencies are built first so clippy runs correctly
$windows_projects = @("pal", "ntreg", "ntstatuserror", "ntuserinfo", "registry")

# projects are in dependency order
$projects = @(
    "tree-sitter-dscexpression",
    "dsc_lib",
    "file_lib",
    "dsc",
    "osinfo",
    "powershellgroup",
    "process",
    "tools/dsctest",
    "tools/test_group_resource",
    "y2j"
    "powershellgroup"
    "wmigroup"
    "resources/brew"
    "tools/dsctest"
)
$pedantic_unclean_projects = @("ntreg")
$clippy_unclean_projects = @("tree-sitter-dscexpression")
$skip_test_projects_on_windows = @("tree-sitter-dscexpression")

if ($IsWindows) {
    $projects += $windows_projects
}

$failed = $false
foreach ($project in $projects) {
    ## Build format_json
    Write-Host -ForegroundColor Cyan "Building $project ... for $architecture"
    try {
        Push-Location "$PSScriptRoot/$project" -ErrorAction Stop

        if ($project -eq 'tree-sitter-dscexpression') {
            ./build.ps1
        }

        if (Test-Path "./Cargo.toml")
        {
            if ($Clippy) {
                if ($clippy_unclean_projects -contains $project) {
                    Write-Verbose -Verbose "Skipping clippy for $project"
                }
                elseif ($pedantic_unclean_projects -contains $project) {
                    Write-Verbose -Verbose "Running clippy for $project"
                    cargo clippy @flags -- -Dwarnings
                }
                else {
                    Write-Verbose -Verbose "Running clippy with pedantic for $project"
                    cargo clippy @flags --% -- -Dwarnings -Dclippy::pedantic
                }
            }
            else {
                cargo build @flags
            }
        }

        if ($LASTEXITCODE -ne 0) {
            $failed = $true
        }

        $binary = Split-Path $project -Leaf

        if ($IsWindows) {
            Copy-Item "$path/$binary.exe" $target -ErrorAction Ignore
        }
        else {
            Copy-Item "$path/$binary" $target -ErrorAction Ignore
        }

        if (Test-Path "./copy_files.txt") {
            Get-Content "./copy_files.txt" | ForEach-Object {
                Copy-Item $_ $target -Force -ErrorAction Ignore
            }
        }

        Copy-Item "*.dsc.resource.json" $target -Force -ErrorAction Ignore

    } finally {
        Pop-Location
    }
}

if ($failed) {
    Write-Host -ForegroundColor Red "Build failed"
    exit 1
}

Copy-Item $PSScriptRoot/tools/add-path.ps1 $target -Force -ErrorAction Ignore

$relative = Resolve-Path $target -Relative
Write-Host -ForegroundColor Green "`nEXE's are copied to $target ($relative)"

# remove the other target in case switching between them
$dirSeparator = [System.IO.Path]::DirectorySeparatorChar
if ($Release) {
    $oldTarget = $target.Replace($dirSeparator + 'release', $dirSeparator + 'debug')
}
else {
    $oldTarget = $target.Replace($dirSeparator + 'debug', $dirSeparator + 'release')
}
$env:PATH = $env:PATH.Replace($oldTarget, '')

$paths = $env:PATH.Split([System.IO.Path]::PathSeparator)
$found = $false
foreach ($path in $paths) {
    if ($path -eq $target) {
        $found = $true
        break
    }
}

# remove empty entries from path
$env:PATH = [string]::Join([System.IO.Path]::PathSeparator, $env:PATH.Split([System.IO.Path]::PathSeparator, [StringSplitOptions]::RemoveEmptyEntries))

if (!$found) {
    Write-Host -ForegroundCOlor Yellow "Adding $target to `$env:PATH"
    $env:PATH += [System.IO.Path]::PathSeparator + $target
}

if ($Test) {
    $failed = $false

    $FullyQualifiedName = @{ModuleName="PSDesiredStateConfiguration";ModuleVersion="2.0.7"}
    if (-not(Get-Module -ListAvailable -FullyQualifiedName $FullyQualifiedName))
    {   "Installing module PSDesiredStateConfiguration 2.0.7"
        Set-PSRepository -Name 'PSGallery' -InstallationPolicy Trusted
        Install-Module PSDesiredStateConfiguration -RequiredVersion 2.0.7
    }

    if (-not(Get-Module -ListAvailable -Name Pester))
    {   "Installing module Pester"
        Set-PSRepository -Name 'PSGallery' -InstallationPolicy Trusted
        Install-Module Pester -WarningAction Ignore
    }

    foreach ($project in $projects) {
        if ($IsWindows -and $skip_test_projects_on_windows -contains $project) {
            Write-Verbose -Verbose "Skipping test for $project on Windows"
            continue
        }

        Write-Host -ForegroundColor Cyan "Testing $project ..."
        try {
            Push-Location "$PSScriptRoot/$project"
            if (Test-Path "./Cargo.toml")
            {
                cargo test

                if ($LASTEXITCODE -ne 0) {
                    $failed = $true
                }
            }
        } finally {
            Pop-Location
        }
    }

    if ($failed) {
        throw "Test failed"
    }

    "PSModulePath is:"
    $env:PSModulePath
    "Pester module located in:"
    (Get-Module -Name Pester -ListAvailable).Path

    # On Windows disable duplicated WinPS resources that break PSDesiredStateConfiguration module
    if ($IsWindows) {
        $a = $env:PSModulePath -split ";" | ? { $_ -notmatch 'WindowsPowerShell' }
        $env:PSModulePath = $a -join ';'

        "Updated PSModulePath is:"
        $env:PSModulePath

        if (-not(Get-Module -ListAvailable -Name Pester))
        {   "Installing module Pester"
            $InstallTargetDir = ($env:PSModulePath -split ";")[0]
            Find-Module -Name 'Pester' -Repository 'PSGallery' | Save-Module -Path $InstallTargetDir
        }

        "Updated Pester module location:"
        (Get-Module -Name Pester -ListAvailable).Path
    }

    Invoke-Pester -ErrorAction Stop
}

$env:RUST_BACKTRACE=1
