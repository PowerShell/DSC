# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

param(
    [switch]$Release,
    [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
    $architecture = 'current',
    [switch]$Clippy,
    [switch]$SkipBuild,
    [ValidateSet('msix','msix-private','msixbundle','tgz','zip')]
    $packageType,
    [switch]$Test,
    [switch]$GetPackageVersion,
    [switch]$SkipLinkCheck,
    [switch]$UseX64MakeAppx,
    [switch]$UseCratesIO,
    [switch]$UpdateLockFile,
    [switch]$Audit
)

if ($GetPackageVersion) {
    $match = Select-String -Path $PSScriptRoot/dsc/Cargo.toml -Pattern '^version\s*=\s*"(?<ver>.*?)"$'
    if ($null -eq $match) {
        throw 'Unable to find version in Cargo.toml'
    }

    return $match.Matches.Groups[1].Value
}

$filesForWindowsPackage = @(
    'dsc.exe',
    'dscecho.exe',
    'echo.dsc.resource.json',
    'assertion.dsc.resource.json',
    'group.dsc.resource.json',
    'powershell.dsc.resource.json',
    'psDscAdapter/',
    'reboot_pending.dsc.resource.json',
    'reboot_pending.resource.ps1',
    'registry.dsc.resource.json',
    'registry.exe',
    'RunCommandOnSet.dsc.resource.json',
    'RunCommandOnSet.exe',
    'windowspowershell.dsc.resource.json',
    'wmi.dsc.resource.json',
    'wmi.resource.ps1'
)

$filesForLinuxPackage = @(
    'dsc',
    'dscecho',
    'echo.dsc.resource.json',
    'assertion.dsc.resource.json',
    'apt.dsc.resource.json',
    'apt.dsc.resource.sh',
    'group.dsc.resource.json',
    'powershell.dsc.resource.json',
    'psDscAdapter/',
    'RunCommandOnSet.dsc.resource.json',
    'runcommandonset'
)

$filesForMacPackage = @(
    'dsc',
    'dscecho',
    'echo.dsc.resource.json',
    'assertion.dsc.resource.json',
    'brew.dsc.resource.json',
    'brew.dsc.resource.sh',
    'group.dsc.resource.json',
    'powershell.dsc.resource.json',
    'psDscAdapter/',
    'RunCommandOnSet.dsc.resource.json',
    'runcommandonset'
)

# the list of files other than the binaries which need to be executable
$filesToBeExecutable = @(
    'apt.dsc.resource.sh',
    'brew.dsc.resource.sh'
)

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

if ($null -ne (Get-Command rustup -ErrorAction Ignore)) {
    $rustup = 'rustup'
} else {
    $rustup = 'echo'
}

if ($null -ne $packageType) {
    $SkipBuild = $true
} else {
    ## Test if Rust is installed
    if (!(Get-Command 'cargo' -ErrorAction Ignore)) {
        Write-Verbose -Verbose "Rust not found, installing..."
        if (!$IsWindows) {
            curl https://sh.rustup.rs -sSf | sh -s -- -y
            $env:PATH += ":$env:HOME/.cargo/bin"
        }
        else {
            Invoke-WebRequest 'https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe' -OutFile 'temp:/rustup-init.exe'
            Write-Verbose -Verbose "Use the default settings to ensure build works"
            & 'temp:/rustup-init.exe' -y
            $env:PATH += ";$env:USERPROFILE\.cargo\bin"
            Remove-Item temp:/rustup-init.exe -ErrorAction Ignore
        }
    }

    $BuildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"

    & $rustup default stable
}

if (!$SkipBuild -and !$SkipLinkCheck -and $IsWindows -and !(Get-Command 'link.exe' -ErrorAction Ignore)) {
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

$configuration = $Release ? 'release' : 'debug'
$flags = @($Release ? '-r' : $null)
if ($architecture -eq 'current') {
    $path = ".\target\$configuration"
    $target = Join-Path $PSScriptRoot 'bin' $configuration
}
else {
    & $rustup target add $architecture
    $flags += '--target'
    $flags += $architecture
    $path = ".\target\$architecture\$configuration"
    $target = Join-Path $PSScriptRoot 'bin' $architecture $configuration
}

if (!$SkipBuild) {
    if (Test-Path $target) {
        Remove-Item $target -Recurse -ErrorAction Ignore
    }
    New-Item -ItemType Directory $target -ErrorAction Ignore > $null

    if ($UseCratesIO) {
        # this will override the config.toml
        Write-Host "Setting CARGO_SOURCE_crates-io_REPLACE_WITH to 'crates-io'"
        ${env:CARGO_SOURCE_crates-io_REPLACE_WITH} = 'CRATESIO'
        $env:CARGO_REGISTRIES_CRATESIO_INDEX = 'sparse+https://index.crates.io/'
    } else {
        Write-Host "Using CFS for cargo source replacement"
        ${env:CARGO_SOURCE_crates-io_REPLACE_WITH} = $null
        $env:CARGO_REGISTRIES_CRATESIO_INDEX = $null

        if ($null -eq (Get-Command 'az' -ErrorAction Ignore)) {
            throw "Azure CLI not found"
        }

        if ($null -ne $env:CARGO_REGISTRIES_POWERSHELL_TOKEN) {
            Write-Host "Using existing token"
        } else {
            Write-Host "Getting token"
            $accessToken = az account get-access-token --query accessToken --resource 499b84ac-1321-427f-aa17-267ca6975798 -o tsv
            if ($LASTEXITCODE -ne 0) {
                Write-Warning "Failed to get access token, use 'az login' first, or use '-useCratesIO' to use crates.io.  Proceeding with anonymous access."
            } else {
                $header = "Bearer $accessToken"
                $env:CARGO_REGISTRIES_POWERSHELL_TOKEN = $header
                $env:CARGO_REGISTRIES_POWERSHELL_CREDENTIAL_PROVIDER = 'cargo:token'
            }
        }
    }

    # make sure dependencies are built first so clippy runs correctly
    $windows_projects = @("pal", "registry", "reboot_pending", "wmi-adapter")
    $macOS_projects = @("resources/brew")
    $linux_projects = @("resources/apt")

    # projects are in dependency order
    $projects = @(
        "tree-sitter-dscexpression",
        "security_context_lib",
        "dsc_lib",
        "dsc",
        "dscecho",
        "osinfo",
        "powershell-adapter",
        "process",
        "runcommandonset",
        "tools/dsctest",
        "tools/test_group_resource",
        "y2j"
    )
    $pedantic_unclean_projects = @()
    $clippy_unclean_projects = @("tree-sitter-dscexpression")
    $skip_test_projects_on_windows = @("tree-sitter-dscexpression")

    if ($IsWindows) {
        $projects += $windows_projects
    }

    if ($IsMacOS) {
        $projects += $macOS_projects
    }

    if ($IsLinux) {
        $projects += $linux_projects
    }

    $failed = $false
    foreach ($project in $projects) {
        ## Build format_json
        Write-Host -ForegroundColor Cyan "Building $project ... for $architecture"
        try {
            Push-Location "$PSScriptRoot/$project" -ErrorAction Stop

            if ($project -eq 'tree-sitter-dscexpression') {
                if ($UpdateLockFile) {
                    cargo generate-lockfile
                }
                else {
                    if ($Audit) {
                        if ($null -eq (Get-Command cargo-audit -ErrorAction Ignore)) {
                            cargo install cargo-audit --features=fix
                        }

                        cargo audit fix
                    }

                    ./build.ps1
                }
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
                    if ($UpdateLockFile) {
                        cargo generate-lockfile
                    }
                    else {
                        if ($Audit) {
                            if ($null -eq (Get-Command cargo-audit -ErrorAction Ignore)) {
                                cargo install cargo-audit --features=fix
                            }

                            cargo audit fix
                        }

                        cargo build @flags
                    }
                }
            }

            if ($LASTEXITCODE -ne 0) {
                $failed = $true
                break
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
                    # if the line contains a '\' character, throw an error
                    if ($_ -match '\\') {
                        throw "copy_files.txt should use '/' as the path separator"
                    }
                    # copy the file to the target directory, creating the directory path if needed
                    $fileCopyPath = $_.split('/')
                    if ($fileCopyPath.Length -gt 1) {
                        $fileCopyPath = $fileCopyPath[0..($fileCopyPath.Length - 2)]
                        $fileCopyPath = $fileCopyPath -join '/'
                        New-Item -ItemType Directory -Path "$target/$fileCopyPath" -Force -ErrorAction Ignore | Out-Null
                    }
                    Copy-Item $_ "$target/$_" -Force -ErrorAction Ignore
                }
            }

            Copy-Item "*.dsc.resource.json" $target -Force -ErrorAction Ignore

            # be sure that the files that should be executable are executable
            if ($IsLinux -or $IsMacOS) {
                foreach ($exeFile in $filesToBeExecutable) {
                    $exePath = "$target/$exeFile"
                    if (test-path $exePath) {
                        chmod +x $exePath
                    }
                }
            }

        } finally {
            Pop-Location
        }
    }

    if ($failed) {
        Write-Host -ForegroundColor Red "Build failed"
        exit 1
    }
}

if (!$Clippy -and !$SkipBuild) {
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
        $env:PATH = $target + [System.IO.Path]::PathSeparator + $env:PATH
    }
}

if ($Test) {
    $failed = $false

    $usingADO = ($null -ne $env:TF_BUILD)
    $repository = 'PSGallery'

    if ($usingADO) {
        $repository = 'CFS'
        if ($null -eq (Get-PSResourceRepository -Name CFS -ErrorAction Ignore)) {
            "Registering CFS repository"
            Register-PSResourceRepository -uri 'https://pkgs.dev.azure.com/powershell/PowerShell/_packaging/powershell/nuget/v2' -Name CFS -Trusted
        }
    }

    if ($IsWindows) {
        # PSDesiredStateConfiguration module is needed for Microsoft.Windows/WindowsPowerShell adapter
        $FullyQualifiedName = @{ModuleName="PSDesiredStateConfiguration";ModuleVersion="2.0.7"}
        if (-not(Get-Module -ListAvailable -FullyQualifiedName $FullyQualifiedName))
        {
            Install-PSResource -Name PSDesiredStateConfiguration -Version 2.0.7 -Repository $repository -TrustRepository
        }
    }

    if (-not(Get-Module -ListAvailable -Name Pester))
    {   "Installing module Pester"
        Install-PSResource Pester -WarningAction Ignore -Repository $repository -TrustRepository
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
            Find-PSResource -Name 'Pester' -Repository $repository | Save-PSResource -Path $InstallTargetDir -TrustRepository
        }

        "Updated Pester module location:"
        (Get-Module -Name Pester -ListAvailable).Path
    }

    Invoke-Pester -ErrorAction Stop
}

function Find-MakeAppx() {
    $makeappx = Get-Command makeappx -CommandType Application -ErrorAction Ignore
    if ($null -eq $makeappx) {
        # try to find
        if (!$UseX64MakeAppx -and $architecture -eq 'aarch64-pc-windows-msvc') {
            $arch = 'arm64'
        }
        else {
            $arch = 'x64'
        }

        $makeappx = Get-ChildItem -Recurse -Path (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin\*\' $arch) -Filter makeappx.exe | Sort-Object FullName -Descending | Select-Object -First 1
        if ($null -eq $makeappx) {
            throw "makeappx not found, please install Windows SDK"
        }
    }

    $makeappx
}

$productVersion = ((Get-Content $PSScriptRoot/dsc/Cargo.toml) -match '^version\s*=\s*') -replace 'version\s*=\s*"(.*?)"', '$1'

if ($packageType -eq 'msixbundle') {
    if (!$IsWindows) {
        throw "MsixBundle is only supported on Windows"
    }

    $packageName = "DSC-$productVersion-Win"
    $makeappx = Find-MakeAppx
    $msixPath = Join-Path $PSScriptRoot 'bin' 'msix'
    & $makeappx bundle /d $msixPath /p "$PSScriptRoot\bin\$packageName.msixbundle"
    return
} elseif ($packageType -eq 'msix' -or $packageType -eq 'msix-private') {
    if (!$IsWindows) {
        throw "MSIX is only supported on Windows"
    }

    if ($architecture -eq 'current') {
        throw 'MSIX requires a specific architecture'
    }

    $isPrivate = $packageType -eq 'msix-private'

    $makeappx = Find-MakeAppx
    $makepri = Get-Item (Join-Path $makeappx.Directory "makepri.exe") -ErrorAction Stop
    $displayName = "DesiredStateConfiguration"
    $isPreview = $productVersion -like '*-*'
    $productName = "DesiredStateConfiguration"
    if ($isPreview) {
        Write-Verbose -Verbose "Preview version detected"
        if ($isPrivate) {
            $productName += "-Private"
        }
        else {
            $productName += "-Preview"
        }
        # save preview number
        $previewNumber = $productVersion -replace '.*?-[a-z]+\.([0-9]+)', '$1'
        # remove label from version
        $productVersion = $productVersion.Split('-')[0]
        # replace revision number with preview number
        $productVersion = $productVersion -replace '(\d+)$', "$previewNumber.0"

        if ($isPrivate) {
            $displayName += "-Private"
        }
        else {
            $displayName += "-Preview"
        }
    }
    Write-Verbose -Verbose "Product version is $productVersion"
    $arch = if ($architecture -eq 'aarch64-pc-windows-msvc') { 'arm64' } else { 'x64' }

    # Appx manifest needs to be in root of source path, but the embedded version needs to be updated
    # cp-459155 is 'CN=Microsoft Windows Store Publisher (Store EKU), O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
    # authenticodeFormer is 'CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
    $releasePublisher = 'CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US'

    $appxManifest = Get-Content "$PSScriptRoot\packaging\msix\AppxManifest.xml" -Raw
    $appxManifest = $appxManifest.Replace('$VERSION$', $ProductVersion).Replace('$ARCH$', $Arch).Replace('$PRODUCTNAME$', $productName).Replace('$DISPLAYNAME$', $displayName).Replace('$PUBLISHER$', $releasePublisher)
    $msixTarget = Join-Path $PSScriptRoot 'bin' $architecture 'msix'
    if (Test-Path $msixTarget) {
        Remove-Item $msixTarget -Recurse -ErrorAction Stop -Force
    }

    New-Item -ItemType Directory $msixTarget > $null
    Set-Content -Path "$msixTarget\AppxManifest.xml" -Value $appxManifest -Force

    foreach ($file in $filesForWindowsPackage) {
        if ((Get-Item "$target\$file") -is [System.IO.DirectoryInfo]) {
            Copy-Item "$target\$file" "$msixTarget\$file" -Recurse -ErrorAction Stop
        } else {
            Copy-Item "$target\$file" $msixTarget -ErrorAction Stop
        }
    }

    # Necessary image assets need to be in source assets folder
    $assets = @(
        'Square150x150Logo'
        'Square64x64Logo'
        'Square44x44Logo'
        'Square44x44Logo.targetsize-48'
        'Square44x44Logo.targetsize-48_altform-unplated'
        'StoreLogo'
    )

    New-Item -ItemType Directory "$msixTarget\assets" > $null
    foreach ($asset in $assets) {
        Copy-Item "$PSScriptRoot\packaging\assets\$asset.png" "$msixTarget\assets" -ErrorAction Stop
    }

    Write-Verbose "Creating priconfig.xml" -Verbose
    & $makepri createconfig /o /cf (Join-Path $msixTarget "priconfig.xml") /dq en-US
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create priconfig.xml"
    }

    Write-Verbose "Creating resources.pri" -Verbose
    Push-Location $msixTarget
    & $makepri new /v /o /pr $msixTarget /cf (Join-Path $msixTarget "priconfig.xml")
    Pop-Location
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create resources.pri"
    }

    Write-Verbose "Creating msix package" -Verbose

    $targetFolder = Join-Path $PSScriptRoot 'bin' 'msix'
    if (Test-Path $targetFolder) {
        Remove-Item $targetFolder -Recurse -ErrorAction Stop -Force
    } else {
        New-Item -ItemType Directory $targetFolder > $null
    }

    $packageName = Join-Path $targetFolder "$productName-$productVersion-$arch.msix"
    & $makeappx pack /o /v /h SHA256 /d $msixTarget /p $packageName
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create msix package"
    }

    Write-Host -ForegroundColor Green "`nMSIX package is created at $packageName"
} elseif ($packageType -eq 'zip') {
    $zipTarget = Join-Path $PSScriptRoot 'bin' $architecture 'zip'
    if (Test-Path $zipTarget) {
        Remove-Item $zipTarget -Recurse -ErrorAction Stop -Force
    }

    New-Item -ItemType Directory $zipTarget > $null

    foreach ($file in $filesForWindowsPackage) {
        if ((Get-Item "$target\$file") -is [System.IO.DirectoryInfo]) {
            Copy-Item "$target\$file" "$zipTarget\$file" -Recurse -ErrorAction Stop
        } else {
            Copy-Item "$target\$file" $zipTarget -ErrorAction Stop
        }
    }

    $packageName = "DSC-$productVersion-$architecture.zip"
    $zipFile = Join-Path $PSScriptRoot 'bin' $packageName
    Compress-Archive -Path "$zipTarget/*" -DestinationPath $zipFile -Force
    Write-Host -ForegroundColor Green "`nZip file is created at $zipFile"
} elseif ($packageType -eq 'tgz') {
    $tgzTarget = Join-Path $PSScriptRoot 'bin' $architecture 'tgz'
    if (Test-Path $tgzTarget) {
        Remove-Item $tgzTarget -Recurse -ErrorAction Stop -Force
    }

    New-Item -ItemType Directory $tgzTarget > $null

    if ($IsLinux) {
        $filesForPackage = $filesForLinuxPackage
    } elseif ($IsMacOS) {
        $filesForPackage = $filesForMacPackage
    } else {
        Write-Error "Unsupported platform for tgz package"
    }

    foreach ($file in $filesForPackage) {
        if ((Get-Item "$target\$file") -is [System.IO.DirectoryInfo]) {
            Copy-Item "$target\$file" "$tgzTarget\$file" -Recurse -ErrorAction Stop
        } else {
            Copy-Item "$target\$file" $tgzTarget -ErrorAction Stop
        }
    }

    $packageName = "DSC-$productVersion-$architecture.tar"
    $tarFile = Join-Path $PSScriptRoot 'bin' $packageName
    tar cvf $tarFile -C $tgzTarget .
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create tar file"
    }
    Write-Host -ForegroundColor Green "`nTar file is created at $tarFile"

    $gzFile = "$tarFile.gz"
    gzip -c $tarFile > $gzFile
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to create gz file"
    }
    Write-Host -ForegroundColor Green "`nGz file is created at $gzFile"
}

$env:RUST_BACKTRACE=1
