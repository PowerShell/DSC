# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

#region Build type definitions
[System.FlagsAttribute()]
enum DscSupportedPlatformOS {
    None    = 0
    Windows = 1
    MacOS   = 2
    Linux   = 4
}

class DscProjectSkipTest {
    [bool] $Linux
    [bool] $MacOS
    [bool] $Windows
}

class DscProjectCopyFiles {
    [string[]] $All
    [string[]] $Linux
    [string[]] $MacOS
    [string[]] $Windows
}

class DscProjectDefinition {
    [string] $Name
    [string] $RelativePath
    [string] $Kind
    [bool]   $IsRust
    [string] $RustPackageName
    [bool]   $ClippyUnclean
    [bool]   $ClippyPedanticUnclean
    [bool]   $SkipTestProject
    [bool]   $OperatingSystemCheck
    [bool]   $TestOnly
    [DscSupportedPlatformOS] $SupportedPlatformOS = 7
    [string[]] $Binaries
    [DscProjectCopyFiles] $CopyFiles
    [DscProjectSkipTest] $SkipTest

    [string[]] ToPackageFlags() {
        if (-not $this.IsRust) {
            return $null
        }

        return @(
            '-p'
            $this.RustPackageName ?? $this.Name
        )
    }

    [string] ToJson([bool]$forBuild) {
        $json = $this.ToData($forBuild) | ConvertTo-Json -Depth 5 -EnumsAsStrings
        return ($json -replace "`r`n", "`n")
    }
    [string] ToJson() {
        return $this.ToJson($false)
    }
    [System.Collections.Specialized.OrderedDictionary] ToData([bool]$forBuild) {
        $data = [ordered]@{
            Name = $this.Name
            Kind = $this.Kind
        }
        if ($forBuild ) {
            $data.RelativePath = $this.RelativePath
        }
        if ($this.SupportedPlatformOS -ne 7) {
            switch ($this.SupportedPlatformOS) {
                Linux {
                    $data.SupportedPlatformOS = 'Linux'
                }
                MacOS {
                    $data.SupportedPlatformOS = 'macOS'
                }
                Windows {
                    $data.SupportedPlatformOS = 'Windows'
                }
            }
        }
        if ($this.IsRust) {
            $data.IsRust = $true
            if (-not [string]::IsNullOrEmpty($this.RustPackageName)) {
                $data.RustPackageName = $this.RustPackageName
            }
            if ($this.ClippyPedanticUnclean) {
                $data.ClippyPedanticUnclean = $true
            }
            if ($this.ClippyUnclean) {
                $data.ClippyUnclean = $true
            }
        }
        if ($null -ne $this.Binaries) {
            $data.Binaries = $this.Binaries
        }
        if ($this.TestOnly) {
            $data.TestOnly = $true
        }
        if ($null -ne $this.SkipTest) {
            $data.SkipTest = [ordered]@{}
            if ($this.SkipTest.Linux) {
                $data.SkipTest.Linux = $this.SkipTest.Linux
            }
            if ($this.SkipTest.MacOS) {
                $data.SkipTest.macOS = $this.SkipTest.macOS
            }
            if ($this.SkipTest.Windows) {
                $data.SkipTest.Windows = $this.SkipTest.Windows
            }
        }
        if ($null -ne $this.CopyFiles) {
            $data.CopyFiles = [ordered]@{}
            if ($this.CopyFiles.All) {
                $data.CopyFiles.All = $this.CopyFiles.All
            }
            if ($this.CopyFiles.Linux) {
                $data.CopyFiles.Linux = $this.CopyFiles.Linux
            }
            if ($this.CopyFiles.MacOS) {
                $data.CopyFiles.macOS = $this.CopyFiles.MacOS
            }
            if ($this.CopyFiles.Windows) {
                $data.CopyFiles.Windows = $this.CopyFiles.Windows
            }
        }

        return $data
    }
    [System.Collections.Specialized.OrderedDictionary] ToData() {
        return $this.ToData($false)
    }
}

class DscProjectPackageFiles {
    [string[]] $Linux
    [string[]] $MacOS
    [string[]] $Windows
    [string[]] $Executable
}

class DscProjectBuildData {
    [DscProjectPackageFiles]$PackageFiles
    [DscProjectDefinition[]]$Projects

    [string] ToJson() {
        $json = $this.ToData() | ConvertTo-Json -Depth 6 -EnumsAsStrings
        return ($json -replace "`r`n", "`n")
    }
    [System.Collections.Specialized.OrderedDictionary] ToData() {
        $data = [ordered]@{
            PackageFiles = [ordered]@{
                Executable = $this.PackageFiles.Executable
                Linux      = $this.PackageFiles.Linux
                macOS      = $this.PackageFiles.MacOS
                Windows    = $this.PackageFiles.Windows
            }
            Projects = @()
        }
        foreach ($project in $this.Projects) {
            $data.Projects += $project.ToData($true)
        }

        return $data
    }
}

class DscArtifactDirectoryPath {
    [string]$BinRoot
    [string]$Bin
    [string]$RustTarget
    [string]$DebTarget
    [string]$RpmTarget
    [string]$MsixBundle
    [string]$MsixTarget
    [string]$ZipTarget
    [string]$TgzTarget
}

#endregion Build type definitions

#region Build data functions
function Import-DscBuildData {
    [CmdletBinding()]
    [OutputType("DscProjectBuildData")]
    param(
        [switch]$RefreshProjects
    )

    begin {
        $buildDataFilePath = Join-Path $PSScriptRoot "data.build.json"
    }

    process {
        if (-not (Test-Path $buildDataFilePath)) {
            throw "Build data file not found: '$buildDataFilePath'"
        }
        $data = Get-Content -Path $buildDataFilePath -Raw
        | ConvertFrom-Json -AsHashtable

        if ($RefreshProjects) {
            [DscProjectDefinition[]]$rootProject = $data.Projects.Where({
                $_.Name -eq 'root'
            }, 'first')[0]
            $data.Projects = $rootProject + (Get-DscProjectData)
        } else {
            $data.Projects = [DscProjectDefinition[]]$data.Projects
        }

        [DscProjectBuildData]$data
    }
}

function Update-DscBuildData {
    [cmdletbinding()]
    param(
        [switch]$PassThru
    )

    begin {
        function Write-ComparisonVerbose {
            [CmdletBinding()]
            param(
                [DscProjectDefinition]$Current,
                [DscProjectDefinition]$New,
                [string]$PropertyDotPath
            )

            begin {
                $name     = $New.Name
                $oldValue = $Current
                $newValue = $New
                foreach ($segment in ($PropertyDotPath -split '.')) {
                    $oldValue = $oldValue.$segment
                    $newValue = $newValue.$segment
                }
            }

            process {
                if ($oldValue -ne $newValue) {
                    Write-Verbose (@(
                        "Updating '$PropertyDotPath' for '$name' from:"
                        "'$oldValue'"
                        "to"
                        "'$newValue'"
                    ) -join " ")
                }
            }
        }
        $propertyDotPaths = @(
            'RelativePath'
            'Kind'
            'IsRust'
            'ClippyUnclean'
            'ClippyPedanticUnclean'
            'SkipTestProject'
            'OperatingSystemCheck'
            'TestOnly'
            'SupportedPlatformOS'
            'Binaries'
            'CopyFiles.All'
            'CopyFiles.Linux'
            'CopyFiles.macOS'
            'CopyFiles.Windows'
            'SkipTest.Linux'
            'SkipTest.macOS'
            'SkipTest.Windows'
        )
        $buildDataFilePath = Join-Path $PSScriptRoot "data.build.json"
    }

    process {
        [DscProjectBuildData]$currentData = Import-DscBuildData
        [DscProjectBuildData]$newData = Import-DscBuildData -RefreshProjects

        foreach ($newProject in $newData.Projects) {
            $current = ($currentData.Projects.Where({
                $_.Name -eq $name
            }, 'first'))
            if ($null -eq $current) {
                Write-Verbose "Adding new project '$($newProject.Name)': $($newProject.ToJson($true))"
                continue
            }
            $comparing = @{
                Current = ($currentData.Projects.Where({
                    $_.Name -eq $name
                }, 'first'))[0]
                New     = $newProject
            }
            if ($VerbosePreference -eq 'Continue') {
                $comparing.Verbose = $true
            }
            foreach ($propertyDotPath in $propertyDotPaths) {
                $comparing.PropertyDotPath = $propertyDotPath
                Write-ComparisonVerbose @comparing
            }
        }

        $rootProject = $currentData.Projects.Where({
            $_.Name -eq 'root'
        }, 'first')[0]
        $newData.Projects = @($rootProject) + $newData.Projects

        $newJson = $newData.ToJson()
        $null = $newJson | Out-File -FilePath $buildDataFilePath -Encoding utf8 -Force

        if ($PassThru) {
            $newData
        }
    }
}

function Get-DscProjectData {
    [cmdletbinding()]
    param(
        [string[]]$Path
    )

    begin {
        if ($null -eq $Path) {
            $Path = $PSScriptRoot
        }
        [System.IO.FileInfo[]]$projectDataFiles = @()
        $repoRootPattern = [regex]::Escape($PSScriptRoot)
        $gciParams = @{
            Recurse = $true
            Filter  = '.project.data.json'
        }
        if (-not $IsWindows) {
            $gciParams.Hidden = $true
        }
    }

    process {
        foreach ($p in $Path) {
            $projectDataFiles += Get-ChildItem -Path $p @gciParams
        }
        foreach ($projectFile in $projectDataFiles) {
            [DscProjectDefinition]$data = Get-Content -Raw -Path $projectFile | ConvertFrom-Json -AsHashtable
            $relativePath = $projectFile.Directory.FullName -replace $repoRootPattern, ''
            $relativePath = $relativePath -replace '\\', '/'
            $relativePath = $relativePath.trim('/')
            $data.RelativePath = $relativePath
            $data
        }
    }
}

function Get-DscCliVersion {
    <#
        .SYNOPSIS
        Returns the current version for the DSC CLI crate.
    #>

    [CmdletBinding()]
    [OutputType([string])]
    param()

    begin {
        $cargoManifestPath = "$PSScriptRoot/dsc/Cargo.toml"
        $pattern = '^version\s*=\s*"(?<ver>.*?)"$'
    }

    process {
        $match = Select-String -Path $cargoManifestPath -Pattern $pattern
        if ($null -eq $match) {
            $PSCmdlet.ThrowTerminatingError('Unable to find DSC CLI version in Cargo.toml')
        }

        $match.Matches.Groups[1].Value
    }
}
#endregion Build data functions

#region Install tools functions
function Get-RustUp {
    <#
        .SYNOPSIS
        Determines whether to use msrustup or public rustup
    #>
    [cmdletbinding()]
    param()

    begin {}

    process {
        $channel = 'stable'
        if ($null -ne (Get-Command msrustup -CommandType Application -ErrorAction Ignore)) {
            Write-Verbose -Verbose "Using msrustup"
            $rustup = 'msrustup'
            $channel = 'ms-prod-1.95'
            if ($architecture -eq 'current') {
                $env:MSRUSTUP_TOOLCHAIN = "$architecture"
            }
        } elseif ($null -ne (Get-Command rustup -CommandType Application -ErrorAction Ignore)) {
                $rustup = 'rustup'
                $env:TESTING_FUNCTION_ENV = "lolwhat"
        }

        return $rustup, $channel
    }
}

function Install-Rust {
    <#
        .SYNOPSIS
        Installs the rust toolchain if not already available.
    #>

    [CmdletBinding()]
    param()

    process {
        if (Test-CommandAvailable -Name 'cargo') {
            Write-Verbose "Rust already installed"
            return
        }

        Write-Verbose -Verbose "Rust not found, installing..."
        if (!$IsWindows) {
            curl https://sh.rustup.rs -sSf | sh -s -- -y
            $env:PATH += ":$env:HOME/.cargo/bin"
        } else {
            Invoke-WebRequest 'https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe' -OutFile 'temp:/rustup-init.exe'
            Write-Verbose -Verbose "Use the default settings to ensure build works"
            & 'temp:/rustup-init.exe' -y
            $env:PATH += ";$env:USERPROFILE\.cargo\bin"
            Remove-Item temp:/rustup-init.exe -ErrorAction Ignore
        }

        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install Rust"
        }
    }
}

function Update-Rust {
    <#
        .SYNOPSIS
        Updates rust if installed, otherwise installs the rust toolchain.
    #>
    [cmdletbinding()]
    param()

    begin {
    }

    process {
        if (-not (Test-CommandAvailable -Name 'cargo')) {
            Install-Rust
            return
        }

        Write-Verbose -Verbose "Rust found, updating..."
        $rustup, $channel = Get-RustUp
        & $rustup toolchain install $channel
        & $rustup default $channel
        & $rustup update
    }
}

function Install-CargoAudit {
    <#
        .SYNOPSIS
        Installs the `cargo audit` command if not already available.
    #>
    [cmdletbinding()]
    param(
        [switch]$UseCFS
    )

    process {
        if (Test-CommandAvailable -Name 'cargo-audit') {
            Write-Verbose "cargo audit already installed."
            return
        }

        Write-Verbose "Installing cargo audit..."
        if ($UseCFS) {
            cargo install cargo-audit --features=fix --config .cargo/config.toml
        } else {
            cargo install cargo-audit --features=fix
        }
    }
}

function Install-CargoLlvmCov {
    <#
        .SYNOPSIS
        Installs `cargo-llvm-cov` if not already available.

        .DESCRIPTION
        Checks whether `cargo-llvm-cov` is installed and installs it if not found. Tries
        `cargo binstall` first (downloads pre-built binary) for speed, then falls back to
        `cargo install` (compiles from source). Also ensures the `llvm-tools-preview` rustup
        component is installed, which is required by cargo-llvm-cov for coverage instrumentation.
    #>
    [CmdletBinding()]
    param(
        [switch]$UseCFS
    )

    process {
        if (Test-CommandAvailable -Name 'cargo-llvm-cov') {
            Write-Verbose -Verbose 'cargo-llvm-cov already installed.'
        } else {
            $installed = $false

            # Try cargo-binstall first (downloads pre-built binary, much faster)
            if (Test-CommandAvailable -Name 'cargo-binstall') {
                Write-Verbose -Verbose 'Installing cargo-llvm-cov via cargo-binstall...'
                cargo binstall --no-confirm cargo-llvm-cov
                if ($LASTEXITCODE -eq 0) {
                    $installed = $true
                } else {
                    Write-Verbose -Verbose 'cargo-binstall failed, falling back to cargo install'
                }
            }

            if (-not $installed) {
                Write-Verbose -Verbose 'Installing cargo-llvm-cov via cargo install (compiling from source)...'
                if ($UseCFS) {
                    cargo install cargo-llvm-cov --config .cargo/config.toml
                } else {
                    cargo install cargo-llvm-cov
                }
                if ($LASTEXITCODE -ne 0) {
                    throw 'Failed to install cargo-llvm-cov'
                }
            }
        }

        Write-Verbose -Verbose 'Ensuring llvm-tools-preview rustup component is installed'
        rustup component add llvm-tools-preview
        if ($LASTEXITCODE -ne 0) {
            throw 'Failed to install llvm-tools-preview rustup component'
        }
    }
}

function Install-TreeSitter {
    <#
        .SYNOPSIS
        Installs the tree-sitter CLI if not already available.
    #>
    [cmdletbinding()]
    param(
        [switch]$UseCFS
    )

    begin {
        $arguments = @(
            'install',
            'tree-sitter-cli',
            '--version', '0.25.10'
        )

        if ($UseCFS) {
            $arguments += '--config'
            $arguments += '.cargo/config.toml'
        }
    }

    process {
        if (Test-CommandAvailable -Name 'tree-sitter') {
            Write-Verbose "tree-sitter already installed."
            return
        }

        Write-Verbose -Verbose "tree-sitter not found, installing..."

        cargo @arguments

        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install tree-sitter-cli"
        }

        # Ensure cargo bin directory is in PATH so tree-sitter can be found
        if (!$IsWindows) {
            $cargoBin = "$env:HOME/.cargo/bin"
            if ($env:PATH -notlike "*$cargoBin*") {
                $env:PATH += ":$cargoBin"
            }
        } else {
            $cargoBin = "$env:USERPROFILE\.cargo\bin"
            if ($env:PATH -notlike "*$cargoBin*") {
                $env:PATH += ";$cargoBin"
            }
        }
    }
}

function Install-WindowsCPlusPlusBuildTools {
    <#
        .SYNOPSIS
        Installs C++ build tools on windows if not already available.
    #>
    [CmdletBinding()]
    param()

    begin {
        $buildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
    }

    process {
        if (-not $IsWindows) {
            return
        }
        if (Test-CommandAvailable -Name 'link.exe') {
            Write-Verbose "C++ build tools already installed."
            return
        }

        if (Test-Path $buildToolsPath) {
            # $linkexe = Find-LinkExe
            # $env:PATH += ";$linkexe"
        } else {
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
    }
}

function Install-Clippy {
    <#
        .SYNOPSIS
        Installs clippy for linting Rust projects.
    #>

    [CmdletBinding()]
    param(
        [switch]$UseCFS,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current'
    )

    begin {
    }

    process {
        Write-Verbose -Verbose "Installing clippy..."
        if ($UseCFS) {
            cargo install clippy --config .cargo/config.toml
        } else {
            $rustup, $channel = Get-RustUp

            if ($rustup -eq 'msrustup') {
                Write-Verbose -Verbose "Clippy is already included with msrustup"
                return
            }

            if ($Architecture -ne 'current') {
                write-verbose -verbose "Installing clippy for $Architecture"
                & $rustup component add clippy --target $Architecture
            } else {
                write-verbose -verbose "Installing clippy for current architecture"
                & $rustup component add clippy
            }
        }
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install clippy"
        }
    }
}

function Install-NodeJS {
    <#
        .SYNOPSIS
        Installs Node.JS at the specified version.
    #>

    [cmdletbinding()]
    param()

    process {
        if (Test-CommandAvailable -Name 'node') {
            Write-Verbose "Node.js already installed."
            return
        }

        Write-Verbose -Verbose "Node.js not found, installing..."
        if ($IsMacOS) {
            if (Test-CommandAvailable -Name 'brew') {
                brew install node@24
            } else {
                Write-Warning "Homebrew not found, please install Node.js manually"
            }
        } elseif ($IsWindows) {
            if (Test-CommandAvailable -Name 'winget') {
                Write-Verbose -Verbose "Using winget to install Node.js"
                winget install OpenJS.NodeJS --accept-source-agreements --accept-package-agreements --source winget --silent
            } else {
                Write-Warning "winget not found, please install Node.js manually"
            }
        } else {
            throw 'Node.js not installed, please install Node.js version 24.x manually'
        }

        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install Node.js"
        }
    }
}

function Install-ProtobufRelease($arch) {
    Write-Verbose -Verbose "Fetching latest Protocol Buffers release info..."
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/protocolbuffers/protobuf/releases/latest"
    $assets = @($release.assets | Where-Object { $_.name -match "protoc-.*-$arch\.zip$" })
    if (-not $assets -or $assets.Count -eq 0) {
        throw "No matching protoc binary found for $arch"
    }
    if ($assets.Count -gt 1) {
        throw "Multiple matching protoc binaries found for $arch"
    }
    $asset = $assets[0]
    $downloadUrl = $asset.browser_download_url
    $tempDir = [System.IO.Path]::GetTempPath()
    $zipPath = Join-Path -Path $tempDir -ChildPath ("protoc-{0}.zip" -f [System.Guid]::NewGuid())

    Write-Host "Downloading protoc from $downloadUrl..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
    $installDir = if ($IsWindows) {
        "$env:USERPROFILE\protoc"
    } else {
        "$env:HOME/protoc"
    }
    if (-not (Test-Path $installDir)) { New-Item -ItemType Directory -Path $installDir | Out-Null }

    Write-Host "Extracting protoc to $installDir..."
    Expand-Archive -Path $zipPath -DestinationPath $installDir -Force

    # Clean up downloaded archive to avoid leaving temporary files behind
    if (Test-Path $zipPath) {
        Remove-Item -Path $zipPath -Force -ErrorAction SilentlyContinue
    }
    $env:PATH = "$installDir" + [System.IO.Path]::DirectorySeparatorChar + "bin" + [System.IO.Path]::PathSeparator + $env:PATH

    Write-Host "Verifying protoc installation..."
    Write-Host (Get-Command protoc | Out-String)
    Write-Host "protoc version: $(protoc --version)"
}

function Install-Protobuf {
    <#
        .SYNOPSIS
        Installs Protobuf for the protoc executable.
    #>

    [cmdletbinding()]
    param()

    process {
        # if ADO, we install the latest version
        if ($null -eq $env:TF_BUILD -and (Test-CommandAvailable -Name 'protoc')) {
            Write-Verbose -Verbose "Protobuf already installed: $(protoc --version)"
            return
        }

        Write-Verbose -Verbose "Protobuf not found, installing..."
        if ($IsMacOS) {
            if (Test-CommandAvailable -Name 'brew') {
                brew install protobuf
            } else {
                Write-Warning "Homebrew not found, please install Protobuf manually"
            }
        } elseif ($IsWindows) {
            if ($env:TF_BUILD) {
                Write-Verbose -Verbose "Running in Azure DevOps, installing from zip"
                $arch = if ([Environment]::Is64BitOperatingSystem) { "win64" } else { "win32" }
                Install-ProtobufRelease -arch $arch
            }
            elseif (Test-CommandAvailable -Name 'winget') {
                Write-Verbose -Verbose "Using winget to install Protobuf"
                winget install Google.Protobuf --accept-source-agreements --accept-package-agreements --source winget --force
                # need to add to PATH
                $protocFolder = "$env:USERPROFILE\AppData\Local\Microsoft\WinGet\Packages\Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe\bin"
                if (Test-Path $protocFolder) {
                    $env:PATH += ";$protocFolder"
                } else {
                    throw "protoc folder not found after installation: $protocFolder"
                }
            } else {
                Write-Warning "winget not found, please install Protobuf manually"
            }
        } else {
            if ($env:TF_BUILD) {
                Write-Verbose -Verbose "Running in Azure DevOps on Linux, installing from zip"
                # check if ARM64 or x64
                $arch = if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq [System.Runtime.InteropServices.Architecture]::Arm64) {
                    "linux-aarch_64"
                } else {
                    "linux-x86_64"
                }
                Install-ProtobufRelease -arch $arch
            } elseif (Test-CommandAvailable -Name 'apt') {
                Write-Verbose -Verbose "Using apt to install Protobuf"
                sudo apt update
                sudo apt install -y protobuf-compiler
                Write-Verbose -Verbose (Get-Command protoc | Out-String)
                Write-Verbose -Verbose "protoc version: $(protoc --version)"
            } else {
                Write-Warning "apt not found, please install Protobuf manually"
            }
        }

        if ($LASTEXITCODE -ne 0) {
            throw "Failed to install Protobuf: $LASTEXITCODE"
        }
    }
}

function Install-PowerShellTestPrerequisite {
    [cmdletbinding()]
    param(
        [switch]$UsingADO
    )

    begin {
        $repository = 'PSGallery'
        if ($usingADO) {
            $repository = 'CFS'
            if ($null -eq (Get-PSResourceRepository -Name CFS -ErrorAction Ignore)) {
                "Registering CFS repository"
                Register-PSResourceRepository -Uri "https://pkgs.dev.azure.com/powershell/PowerShell/_packaging/PowerShellGalleryMirror/nuget/v3/index.json" -Name CFS -Trusted
            }
        }
    }

    process {
        if ($IsWindows) {
            # PSDesiredStateConfiguration module is needed for Microsoft.Windows/WindowsPowerShell adapter
            $FullyQualifiedName = @{ModuleName="PSDesiredStateConfiguration";ModuleVersion="2.0.8"}
            if (-not(Get-Module -ListAvailable -FullyQualifiedName $FullyQualifiedName))
            {
                Install-PSResource -Name PSDesiredStateConfiguration -Version 2.0.8 -Repository $repository -TrustRepository
            }
        }

        if (-not(Get-Module -ListAvailable -Name Pester)){
            Write-Verbose "Installing module 'Pester'"
            Install-PSResource Pester -WarningAction Ignore -Repository $repository -TrustRepository
        }

        if (-not (Get-Module -ListAvailable -Name YaYaml)) {
            Write-Verbose "Installing module 'YaYaml'"
            Install-PSResource YaYaml -WarningAction Ignore -Repository $repository -TrustRepository
        }

        if (-not (Get-Module -ListAvailable -Name PSToml)) {
            Write-Verbose "Installing module 'PSToml'"
            Install-PSResource PSToml -WarningAction Ignore -Repository $repository -TrustRepository
        }
    }
}

#endregion Install tools functions

#region    Environment setup utility functions
function Set-CargoEnvironment {
    <#
        .SYNOPSIS
        Defines environment variables for Cargo to enable retrieving crates.
    #>
    [cmdletbinding()]
    param(
        [switch]$UseCFS
    )

    process {
        if ($UseCFS) {
            Write-Host "Using CFS for cargo source replacement"
            ${env:CARGO_SOURCE_crates-io_REPLACE_WITH} = $null
            $env:CARGO_REGISTRIES_CRATESIO_INDEX = $null
        } else {
            # this will override the config.toml
            Write-Host "Setting CARGO_SOURCE_crates-io_REPLACE_WITH to 'crates-io'"
            ${env:CARGO_SOURCE_crates-io_REPLACE_WITH} = 'CRATESIO'
            $env:CARGO_REGISTRIES_CRATESIO_INDEX = 'sparse+https://index.crates.io/'
        }
    }
}

function Get-RustEnvironment {
    [CmdletBinding()]
    [OutputType([hashtable])]
    param()

    process {
        [hashtable]$currentRustEnvironment = @{}
        foreach ($envVar in (Get-ChildItem -Path 'Env:\RUST*')) {
            $currentRustEnvironment[$envVar.Name] = $envVar.Value
        }
        if ($currentRustEnvironment.Keys.Count -gt 0) {
            $lines = @(
                "Current Rust environment variables:"
            )
            $lines += $currentRustEnvironment.Keys | ForEach-Object -Process {
                "$_ = '$($currentRustEnvironment[$_])'"
            }
            Write-Verbose ($lines -join "`n`t")
        } else {
            Write-Verbose "No Rust environment variables defined"
        }
        $currentRustEnvironment
    }
}

function Set-RustEnvironment {
    [cmdletbinding()]
    param(
        [switch]$CacheRustBuild
    )

    process {
        Write-Verbose "Caching current rust environment variables..."
        [hashtable]$currentRustEnvironment = Get-RustEnvironment

        if ($VerbosePreference -eq 'Continue') {
            Write-Verbose "Running verbose, setting RUSTC_LOG "
            # $env:RUSTC_LOG='rustc_codegen_ssa::back::link=info'
        } else {
            Write-Verbose "Disabling RUSTC_LOG for a quieter build"
            $env:RUSTC_LOG = $null
        }

        if ($CacheRustBuild) {
            Enable-RustBuildCaching > $null
        }

        $currentRustEnvironment
    }
}

function Reset-RustEnvironment {
    [cmdletbinding()]
    param(
        [Parameter(Mandatory)]
        [hashtable]$PriorEnvironment
    )

    process {
        Write-Verbose "Resetting Rust environment..."
        $currentEnvironment = Get-RustEnvironment
        foreach ($envVar in $currentEnvironment.Keys) {
            if ($envVar -in $PriorEnvironment.Keys) {
                $value = $PriorEnvironment[$envVar]
                Write-Verbose "Resetting $envVar to '$value'"
                Set-Item -Path "Env:\$envVar" -Value $value
            } else {
                Write-Verbose "Removing $envVar"
                Remove-Item -path "Env:\$envVar"
            }
        }
        Write-Verbose "Reset rust environment"
        Get-RustEnvironment > $null
    }
}

function Enable-RustBuildCaching {
    [cmdletbinding()]
    [OutputType([void])]
    param(
        [string]$CacheAppName = 'sccache',
        [switch]$Force
    )

    begin {
        [string]$cacheApp = (Get-Command $CacheAppName -ErrorAction Ignore).Source
    }

    process {
        Write-Verbose "Checking whether caching app '$CacheAppName' can be used for Rust builds..."
        if ([string]::IsNullOrEmpty($cacheApp)) {
            $message = "$CacheAppName isn't available on this system."
            if ($Force) {
                throw $message
            }
            Write-Verbose $message
            return
        }

        if (-not [string]::IsNullOrEmpty($env:RUSTC_WRAPPER)) {
            Write-Verbose (@(
                "Rust already configured to use a caching wrapper:"
                "RUSTC_WRAPPER = '$($env:RUSTC_WRAPPER)'"
            ) -join "`n`t")
            if (-not $Force) {
                return
            }
            Write-Verbose "Overriding current caching wrapper with '$cacheApp'"
        }

        $env:RUSTC_WRAPPER = $cacheApp
        Write-Verbose (@(
            "Enabled use of '$CacheAppName' for Rust builds:"
            "RUSTC_WRAPPER = '$($env:RUSTC_WRAPPER)'"
        ) -join "`n`t")
    }
}

function Disable-RustSccache {
    [cmdletbinding()]
    param()

    process {
        if ([string]::IsNullOrEmpty($env:RUSTC_WRAPPER)) {
            Write-Verbose "No caching application defined for Rust"
            return
        }
        Write-Verbose "Disabling use of '$($env:RUSTC_WRAPPER)' for caching Rust builds."
        $env:RUSTC_WRAPPER = $null
    }
}

function Get-CargoMetadata {
    [CmdletBinding()]
    param()

    begin {
        $json = cargo metadata --format-version 1
        $data = $json | ConvertFrom-Json -AsHashtable
    }

    process {
        $data.DscMembers = $data.workspace_members | ForEach-Object {
            $name, $version = ($_ -split '/')[-1] -split '#'
            [pscustomobject]@{
                Name    = $name
                Version = $version
            }
        }
    }

    end {
        $data
    }
}

function Set-DefaultWorkspaceMemberGroup {
    [CmdletBinding()]
    param(
        [ValidateSet('All', 'Linux', 'macOS', 'Windows')]
        [string]
        $MemberGroup = 'All',
        [string]$CargoContent
    )

    begin {
        $cargoFilePath = Join-Path $PSScriptRoot -ChildPath 'Cargo.toml'
        if ([string]::IsNullOrEmpty($CargoContent)) {
            $CargoContent = Get-Content $cargoFilePath -Raw
        }
        $params = @{
            MemberGroup  = $MemberGroup
            CargoContent = $CargoContent
        }
        $defaultMembers = Get-DefaultWorkspaceMemberGroup -CargoContent $CargoContent
        $members = Get-WorkspaceMemberGroup @params

        $findPattern = @(
            '(?ms)' # Flags
            '^' # anchor
            '(?<prefix>default-members\s+=\s+\[)'
            '(?<members>.+?)'
            '(?<suffix>\])'
        ) -join ''
        $replacePattern = @(
            '${prefix}'
            $members
            '${suffix}'
        ) -join ''
    }

    process {
        if ($members -eq $defaultMembers) {
            Write-Verbose "Default workspace already set to '$MemberGroup': [$members]"
            return
        }

        Write-Verbose "Setting default workspace members to '$MemberGroup': [$members]"

        $CargoContent -replace $findPattern, $replacePattern
        | Out-File -FilePath $cargoFilePath -Encoding utf8 -Force -NoNewline
    }
}

function Reset-DefaultWorkspaceMemberGroup {
    [CmdletBinding()]
    param()

    process {
        Set-DefaultWorkspaceMemberGroup -MemberGroup All
    }
}

function Get-DefaultWorkspaceMemberGroup {
    [CmdletBinding()]
    param(
        [string]$CargoContent
    )

    begin {
        $cargoFilePath = Join-Path $PSScriptRoot -ChildPath 'Cargo.toml'
        if ([string]::IsNullOrEmpty($CargoContent)) {
            $CargoContent = Get-Content $cargoFilePath -Raw
        }
        $pattern   = '(?ms)^default-members = \[(?<members>.+?)\]'
    }

    process {
        [regex]::Match($CargoContent, $pattern).Groups[0].Groups['members'].Value
    }

}

function Get-WorkspaceMemberGroup {
    [cmdletbinding()]
    param(
        [ValidateSet('All', 'Linux', 'macOS', 'Windows')]
        [string]$MemberGroup = 'All',
        [string]$CargoContent
    )

    begin {
        if ([string]::IsNullOrEmpty($CargoContent)) {
            $CargoContent = Get-Content $PSScriptRoot/Cargo.toml -Raw
        }
        $allMembersPattern   = '(?ms)^members = \[(?<members>.+?)\]'
        $groupMemberspattern = '(?ms)^{0} = \[(?<members>.+?)\]' -f $MemberGroup
    }

    process {
        $pattern = ($MemberGroup -eq 'All') ? $allMembersPattern : $groupMemberspattern
        [regex]::Match($CargoContent, $pattern).Groups[0].Groups['members'].Value
    }
}

function Get-ArtifactDirectoryPath {
    [CmdletBinding()]
    [OutputType('DscArtifactDirectoryPath')]
    param(
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $Architecture = 'current',
        [switch]$Release
    )

    begin {
        $configuration = $Release ? 'release' : 'debug'
    }
    process {
        if ($Architecture -eq 'current') {
            return [DscArtifactDirectoryPath]@{
                BinRoot    = Join-Path $PSScriptRoot 'bin'
                Bin        = Join-Path $PSScriptRoot 'bin' $configuration
                RustTarget = $env:CARGO_TARGET_DIR ?? (Join-Path $PSScriptRoot 'target' $configuration)
                MsixBundle = Join-Path $PSScriptRoot 'bin' 'msix'
            }
        }

        [DscArtifactDirectoryPath]@{
            BinRoot    = Join-Path $PSScriptRoot 'bin'
            Bin        = Join-Path $PSScriptRoot 'bin' $Architecture $configuration
            RustTarget = $env:CARGO_TARGET_DIR ?? (Join-Path $PSScriptRoot 'target' $Architecture $configuration)
            DebTarget  = Join-Path $PSScriptRoot 'bin' $Architecture 'deb'
            RpmTarget  = Join-Path $PSScriptRoot 'bin' $Architecture 'rpm'
            MsixBundle = Join-Path $PSScriptRoot 'bin' 'msix'
            MsixTarget = Join-Path $PSScriptRoot 'bin' $Architecture 'msix'
            ZipTarget  = Join-Path $PSScriptRoot 'bin' $Architecture 'zip'
            TgzTarget  = Join-Path $PSScriptRoot 'bin' $Architecture 'tgz'
        }
    }
}

function Test-CommandAvailable {
    <#
        .SYNOPSIS
        Checks whether a given command is available.
    #>

    [cmdletbinding()]
    [OutputType([bool])]
    param (
        [string]$Name
    )

    end {
        if (Get-Command $Name -ErrorAction Ignore) {
            $true
        } else {
            $false
        }
    }
}

function Find-LinkExe {
    [cmdletbinding()]
    [OutputType([string])]
    param()

    begin {
        $buildToolsPath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC"
    }

    process {
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
}

function Update-PathEnvironment {
    [CmdletBinding()]
    param(
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current',
        [switch]$Release
    )

    begin {
        $binDirectory = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        | Select-Object -ExpandProperty Bin
        $dirSeparator = [System.IO.Path]::DirectorySeparatorChar
    }

    process {
        # remove the other target in case switching between them
        if ($Release) {
            $oldBinDirectory = $binDirectory.Replace($dirSeparator + 'release', $dirSeparator + 'debug')
        }
        else {
            $oldBinDirectory = $binDirectory.Replace($dirSeparator + 'debug', $dirSeparator + 'release')
        }
        $env:PATH = $env:PATH.Replace($oldBinDirectory, '')
        # Find target in path
        $paths = $env:PATH.Split([System.IO.Path]::PathSeparator)
        $found = $false
        foreach ($path in $paths) {
            if ($path -eq $binDirectory) {
                $found = $true
                break
            }
        }
        # remove empty entries from path
        $env:PATH = [string]::Join([System.IO.Path]::PathSeparator, $env:PATH.Split([System.IO.Path]::PathSeparator, [StringSplitOptions]::RemoveEmptyEntries))

        if (!$found) {
            Write-Host -ForegroundColor Yellow "Adding $binDirectory to `$env:PATH"
            $env:PATH = $binDirectory + [System.IO.Path]::PathSeparator + $env:PATH
        }
    }
}

function Find-MakeAppx {
    [CmdletBinding()]
    param(
        # When packaging in OneBranch, the MSIX is created on an x64 image for
        # the arm64 package, and our tooling expects to use the architecture
        # passed, so we have to override it here. It may be possible to
        # workaround this in another way, but deferring further investigation.
        [switch]$UseX64MakeAppx
    )

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

function Invoke-NativeCommand($cmd) {
    Invoke-Expression $cmd
    if ($LASTEXITCODE -ne 0) {
        throw "Command $cmd failed with exit code $LASTEXITCODE"
    }
}
#endregion Environment setup utility functions

#region Test-ShouldSkip functions
function Test-ShouldSkipFileForArchitecture {
    <#
        .SYNOPSIS
        Determines whether to skip copying build files by platform and architecture.
    #>
    [CmdletBinding()]
    param(
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $architecture = 'current',
        [DscSupportedPlatformOS]$OperatingSystem
    )

    begin {
        $windowsArchitectures = @(
            'current'
            'aarch64-pc-windows-msvc'
            'x86_64-pc-windows-msvc'
        )
        $linuxArchitectures = @(
            'current'
            'aarch64-unknown-linux-gnu'
            'aarch64-unknown-linux-musl'
            'x86_64-unknown-linux-gnu'
            'x86_64-unknown-linux-musl'
        )
        $macOSArchitectures = @(
            'current'
            'aarch64-apple-darwin'
            'x86_64-apple-darwin'
        )
    }

    process {
        if ($OperatingSystem -eq [DscSupportedPlatformOS]::Linux) {
            if ($architecture -in $linuxArchitectures -and $IsLinux) {
                return $false
            }
            return $true
        }
        if ($OperatingSystem -eq [DscSupportedPlatformOS]::MacOS) {
            if ($architecture -in $macOSArchitectures -and $IsMacOS) {
                return $false
            }
            return $true
        }
        if ($OperatingSystem -eq [DscSupportedPlatformOS]::Windows) {
            if ($architecture -in $windowsArchitectures -and $IsWindows) {
                return $false
            }
            return $true
        }
        Write-Warning "Couldn't determine whether to skip for $architecture"
        return $true
    }
}

function Test-ShouldSkipProject {
    <#
        .SYNOPSIS
        Determines whether to entirely skip a project by operating system and target architecture.
    #>
    [CmdletBinding()]
    param(
        [DscProjectDefinition]$Project,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current'
    )

    begin {}

    process {
        if ($Architecture -eq 'current') {
            if ($Project.SupportedPlatformOS -eq [DscSupportedPlatformOS]::Windows) {
                return -not $IsWindows
            } elseif ($Project.SupportedPlatformOS -eq [DscSupportedPlatformOS]::MacOS) {
                return -not $IsMacOS
            } elseif ($Project.SupportedPlatformOS -eq [DscSupportedPlatformOS]::Linux) {
                return -not $IsLinux
            }
        }
        return $false
    }
}

function Test-ShouldSkipProjectTesting {
    [CmdletBinding()]
    param(
        [DscProjectDefinition]$Project
    )

    begin {}

    process {
        $shouldSkip = $false
        if ($Project.SkipTest.Linux -and $IsLinux) {
            $shouldSkip = $true
        }
        if ($Project.SkipTest.macOS -and $IsMacOS) {
            $shouldSkip = $true
        }
        if ($Project.SkipTest.Windows -and $IsWindows) {
            $shouldSkip = $true
        }
        return $shouldSkip
    }
}
#endregion Test-ShouldSkip functions

#region    Build project functions
function Export-GrammarBinding {
    [CmdletBinding()]
    param(
        [DscProjectDefinition[]]$Project
    )

    begin {
        if (Test-Path Env:\TREE_SITTER_VERBOSE) {
            $hasPriorSetting = $true
            $priorSetting = $env:TREE_SITTER_VERBOSE
        }

        $env:TREE_SITTER_VERBOSE = 1
        Write-Verbose "Searching for grammar projects..."
        $grammarProjects = $Project | Where-Object -FilterScript { $_.Kind -eq 'Grammar'}
        Write-Verbose "Found grammar projects: [`n`t$($grammarProjects.Name -join "`n`t")`n]"
    }

    process {
        foreach ($grammar in $grammarProjects) {
            Write-Verbose "Exporting grammar binding for '$($grammar.Name)'"
            try {
                Push-Location $grammar.RelativePath
                Invoke-NativeCommand 'tree-sitter init --update'
                Invoke-NativeCommand 'tree-sitter generate --build'
                Invoke-NativeCommand 'tree-sitter test'
            } finally {
                Pop-Location
            }

        }
    }

    clean {
        if ($hasPriorSetting) {
            $env:TREE_SITTER_VERBOSE = $priorSetting
        }
    }
}

function Test-Clippy {
    [CmdletBinding()]
    param(
        [DscProjectDefinition[]]$Project,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$NoModifyWorkspace
    )

    begin {
        $flags = @($Release ? '-r' : $null)
        if ($Architecture -ne 'current') {
            $flags += '--target'
            $flags += $Architecture
            $memberGroup = if ($Architecture -match 'linux') {
                'Linux'
            } elseif ($Architecture -match 'darwin') {
                'macOS'
            } elseif ($Architecture -match 'windows') {
                'Windows'
            } else {
                throw "Unsupported architecture '$Architecture'"
            }
        } else {
            $memberGroup = if ($IsLinux) {
                'Linux'
            } elseif ($IsMacOS) {
                'macOS'
            } elseif ($IsWindows) {
                'Windows'
            }
        }
        $workspaceParams = @{
            MemberGroup = $memberGroup
        }
        if ($VerbosePreference -eq 'Continue') {
            $workspaceParams.Verbose = $true
        }
        if (-not $NoModifyWorkspace) {
            Set-DefaultWorkspaceMemberGroup @workspaceParams
        }
    }

    process {
        $clippyFlags = @(
            '--%'
            '--'
            '-Dwarnings' # Treat warnings as errors
            '--no-deps'  # Only lint DSC projects, not dependencies
        )
        # Collect projects to lint into two groups:
        # - Normal projects get linting without pedantic checks
        # - Pedantic projects get stricter linting
        [DscProjectDefinition[]]$normalProjects = @()
        [DscProjectDefinition[]]$pedanticProjects = @()
        foreach ($p in $Project) {
            if (
                -not $p.IsRust -or
                $p.ClippyUnclean -or
                (Test-ShouldSkipProject -Project $p -Architecture $Architecture)
            ) {
                continue
            }

            if ($p.ClippyPedanticUnclean) {
                $pedanticProjects += $p
            } else {
                $normalProjects += $p
            }
        }
        $normalProjectsExitCode   = 0
        $pedanticProjectsExitCode = 0
        if ($normalProjects.count -gt 0) {
            [string[]]$projectFlags = $normalProjects.ToPackageFlags()
            Write-Verbose "Linting rust projects with clippy: $($normalProjects.Name | ConvertTo-Json)"
            Write-Verbose "Invoking clippy: cargo clippy $flags $projectFlags $clippyFlags"
            cargo clippy @flags @projectFlags @clippyFlags

            if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
                $normalProjectsExitCode = $LASTEXITCODE
            }
        }
        if ($pedanticProjects.count -gt 0) {
            [string[]]$projectFlags = $pedanticProjects.ToPackageFlags()
            $clippyFlags += '-Dclippy::pedantic'
            Write-Verbose "Linting rust projects with clippy (pedantic): $($pedanticProjects.Name | ConvertTo-Json)"
            Write-Verbose "Invoking clippy: cargo clippy $flags $projectFlags $clippyFlags"
            cargo clippy @flags @projectFlags @clippyFlags

            if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
                $pedanticProjectsExitCode = $LASTEXITCODE
            }
        }
        if ($normalProjectsExitCode -or $pedanticProjectsExitCode) {
            throw "Clippy failed for at least one project"
        }
    }

    clean {
        if (-not $NoModifyWorkspace) {
            Reset-DefaultWorkspaceMemberGroup
        }
    }
}

function Build-RustProject {
    [CmdletBinding()]
    param(
        [DscProjectDefinition[]]$Project,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$Clean,
        [switch]$UpdateLockFile,
        [switch]$Audit,
        [switch]$Clippy
    )

    begin {
        $flags = @($Release ? '-r' : $null)
        if ($Architecture -ne 'current') {
            $flags += '--target'
            $flags += $Architecture
            $memberGroup = if ($Architecture -match 'linux') {
                'Linux'
            } elseif ($Architecture -match 'darwin') {
                'macOS'
            } elseif ($Architecture -match 'windows') {
                'Windows'
            } else {
                throw "Unsupported architecture '$Architecture'"
            }
        } else {
            $memberGroup = if ($IsLinux) {
                'Linux'
            } elseif ($IsMacOS) {
                'macOS'
            } elseif ($IsWindows) {
                'Windows'
            }
        }
        $workspaceParams = @{
            MemberGroup = $memberGroup
        }
        if ($VerbosePreference -eq 'Continue') {
            $workspaceParams.Verbose = $true
        }
        Set-DefaultWorkspaceMemberGroup @workspaceParams
    }

    process {
        if ($UpdateLockFile) {
            return cargo generate-lockfile
        }

        if ($Audit) {
            Install-CargoAudit
            cargo audit fix
        }

        if ($Clean) {
            cargo clean
        }

        if ($Clippy) {
            $clippyParams = @{
                Project = $Project
                Architecture = $Architecture
                Release = $Release
                NoModifyWorkspace = $true
            }
            Test-Clippy @clippyParams
        }

        $members = Get-DefaultWorkspaceMemberGroup
        Write-Verbose -Verbose "Building rust projects: [$members]"
        Write-Verbose "Invoking cargo:`n`tcargo build $flags"
        cargo build @flags

        if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
            throw "Last exit code is $LASTEXITCODE, build failed for at least one project"
        }
    }

    clean {
        Reset-DefaultWorkspaceMemberGroup
    }
}

function Copy-BuildArtifact {
    [CmdletBinding()]
    param(
        [DscProjectDefinition[]]$Project,
        [string[]]$ExecutableFile,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$Clean
    )

    begin {
        $artifactDirectory = Get-ArtifactDirectoryPath -Release:$Release -Architecture $Architecture
        $copyParams = @{
            Destination = $artifactDirectory.bin
            ErrorAction = 'Ignore'
            # Verbose     = $true
            Force       = $true
        }
        if (-not (Test-Path -Path $artifactDirectory.bin)) {
            Write-Verbose "Creating output directory for artifacts at: '$($artifactDirectory.bin)'"
            New-Item -Path $artifactDirectory.bin -ItemType Directory -Force
        }
        if ($Clean) {
            Write-Verbose "Cleaning output directory for artifacts ('$($artifactDirectory.bin)')"
            Remove-Item -Path "$($artifactDirectory.bin)/*" -Recurse -Force
        }
    }

    process {
        foreach ($p in $Project) {
            # SKip projects for non-current architecture if needed
            if (Test-ShouldSkipProject -Project $p -Architecture $Architecture) {
                Write-Verbose "Skipping '$($p.Name)' project build artifacts because OS is not $($p.SupportedPlatformOS)"
                continue
            }

            if ($null -eq $p.CopyFiles -and $null -eq $p.Binaries) {
                continue
            }

            Write-Verbose "Copying '$($p.Name)' project build artifacts..."

            # First, copy compiled binary files
            if ($p.Binaries) {
                Write-Verbose "Copying '$($p.Name)' project binaries..."
            }
            foreach ($binary in $p.Binaries) {
                $gciParams = @{
                    Path = Join-Path $artifactDirectory.RustTarget '*'
                    File = $true
                    Include = @($binary, "$binary.pdb", "$binary.exe")
                }
                $binaryFiles = Get-ChildItem @gciParams
                foreach ($file in $binaryFiles) {
                    Write-Verbose "Copying '$($p.Name)' binary file: '$($file.Name)'"
                    $file | Copy-Item @copyParams -Destination $artifactDirectory.Bin
                }

            }
            if ($null -eq $p.CopyFiles) {
                continue
            }
            Write-Verbose "Copying '$($p.Name)' project files..."
            # Next, copy other files
            foreach ($file in $p.CopyFiles.All) {
                $source = Join-Path $PSScriptRoot $p.RelativePath $file
                $source = Resolve-Path $source
                $destination = $artifactDirectory.Bin
                if ($file -match '/') {
                    $segments = ($file -split '/' | Select-Object -SkipLast 1)

                    $destination = Join-Path $destination $segments
                }
                if (-not (Test-Path -Path $destination)) {
                    New-Item -Path $destination -ItemType Directory -Force
                }
                Write-Verbose "Copying '$($p.Name)' project file: '$file'`n`tSource: '$source'`n`tDestination: '$destination'"
                $source | Copy-Item @copyParams -Destination $destination
            }
            foreach ($platform in @('Linux', 'macOS', 'Windows')) {
                $files = $p.CopyFiles.$platform
                if ($null -eq $files) {
                    continue
                }
                if (Test-ShouldSkipFileForArchitecture -Architecture $Architecture -OperatingSystem $platform) {
                    Write-Verbose "Skipping copying '$($p.Name)' project files for $platform"
                    continue
                }
                Write-Verbose "Copying '$($p.Name)' project files for $platform"
                foreach ($file in $p.CopyFiles.$platform) {
                    $source = Join-Path $PSScriptRoot $p.RelativePath $file
                    $source = Resolve-Path $source
                    $destination = $artifactDirectory.Bin
                    if ($file -match '/') {
                        $segments = ($file -split '/' | Select-Object -SkipLast 1)

                        $destination = Join-Path $destination $segments
                    }
                    if (-not (Test-Path -Path $destination)) {
                        New-Item -Path $destination -ItemType Directory -Force
                    }
                    Write-Verbose "Copying '$($p.Name)' project file: '$file'`n`tSource: '$source'`n`tDestination: '$destination'"
                    $source | Copy-Item @copyParams -Destination $destination
                }
            }
        }

        foreach ($file in $ExecutableFile) {
            $filePath = Join-Path $artifactDirectory.Bin $file
            if (-not (Test-Path $filePath)) {
                continue
            }
            Write-Verbose "Marking '$file' as executable..."
            chmod +x $filePath
            Write-Verbose "Marked '$filePath' as executable."
        }
    }
}
#endregion Build project functions

#region    Documenting project functions
function Export-RustDocs {
    [CmdletBinding()]
    param(
        [DscProjectDefinition[]]$Project,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$IncludeDependencies
    )

    begin {
        $flags = @($Release ? '-r' : $null)
        if ($Architecture -ne 'current') {
            $flags += '--target'
            $flags += $Architecture
        } else {
            $memberGroup = if ($IsLinux) {
                'Linux'
            } elseif ($IsMacOS) {
                'macOS'
            } elseif ($IsWindows) {
                'Windows'
            }
            Set-DefaultWorkspaceMemberGroup -MemberGroup $memberGroup
        }
        if (-not $IncludeDependencies) {
            $flags += '--no-deps'
        }
    }

    process {
        $members = Get-DefaultWorkspaceMemberGroup
        Write-Verbose -Verbose "Exporting documentation for rust projects: [$members]"
        cargo doc @flags

        if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
            Write-Error "Last exit code is $LASTEXITCODE, 'cargo doc' failed"
        }
    }

    clean {
        Reset-DefaultWorkspaceMemberGroup
    }
}
#endregion Documenting project functions

#region    Code coverage functions
function Get-ChangedRustFile {
    <#
        .SYNOPSIS
        Returns the list of Rust files changed between two commits.

        .DESCRIPTION
        Uses `git diff` to identify `.rs` files that were added, copied, modified, or renamed
        between the specified base and head commits.

        .PARAMETER BaseSha
        The base commit SHA to compare from.

        .PARAMETER HeadSha
        The head commit SHA to compare to.

        .OUTPUTS
        System.String[]
        An array of changed `.rs` file paths, or an empty array if none were changed.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$BaseSha,

        [Parameter(Mandatory)]
        [string]$HeadSha
    )

    process {
        $changedFiles = git diff --name-only --diff-filter=ACMR "$BaseSha..$HeadSha" -- '*.rs'
        if ($LASTEXITCODE -ne 0) {
            Write-Warning "Failed to detect changed files between $BaseSha and $HeadSha"
            return @()
        }

        $result = @($changedFiles | Where-Object { $_ })
        Write-Verbose -Verbose "Found $($result.Count) changed Rust file(s)"
        return $result
    }
}

function Initialize-CodeCoverage {
    <#
        .SYNOPSIS
        Prepares the workspace for code coverage instrumentation using cargo-llvm-cov.

        .DESCRIPTION
        Installs cargo-llvm-cov if needed and cleans any prior coverage artifacts from the
        workspace. After initialization, call Set-LlvmCovEnvironment to set the environment
        variables that make normal `cargo build` and `cargo test` invocations produce
        instrumented binaries and write profraw data.
    #>
    [CmdletBinding()]
    param(
        [switch]$UseCFS
    )

    process {
        $verboseFlag = @{}
        if ($VerbosePreference -eq 'Continue') {
            $verboseFlag.Verbose = $true
        }

        Install-CargoLlvmCov -UseCFS:$UseCFS @verboseFlag

        Write-Verbose -Verbose 'Cleaning previous coverage artifacts'
        cargo llvm-cov clean --workspace
        if ($LASTEXITCODE -ne 0) {
            Write-Warning 'Failed to clean previous coverage artifacts, continuing anyway'
        }
    }
}

function Set-LlvmCovEnvironment {
    <#
        .SYNOPSIS
        Sets the environment variables required by cargo-llvm-cov for instrumented builds.

        .DESCRIPTION
        Parses the output of `cargo llvm-cov show-env` and sets the corresponding
        environment variables (LLVM_PROFILE_FILE, RUSTC_WRAPPER, CARGO_LLVM_COV, etc.)
        in the current process. This enables a normal `cargo build` to produce
        instrumented binaries, and allows externally invoked instrumented binaries
        (such as during Pester tests) to write profraw data to a location that
        `cargo llvm-cov report` can discover.

        .OUTPUTS
        System.Collections.Hashtable — Prior values of the modified environment variables
        so they can be restored with Reset-LlvmCovEnvironment.
    #>
    [CmdletBinding()]
    [OutputType([hashtable])]
    param()

    process {
        $showEnvOutput = cargo llvm-cov show-env 2>&1
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to retrieve cargo-llvm-cov environment: $showEnvOutput"
        }

        $priorValues = @{}
        foreach ($line in $showEnvOutput) {
            if ($line -match '^([A-Z_][A-Z0-9_]+)=(.*)$') {
                $name = $Matches[1]
                # CARGO_LLVM_COV_SHOW_ENV is an output-only flag that tells
                # cargo-llvm-cov to print env and exit; do not propagate it.
                if ($name -eq 'CARGO_LLVM_COV_SHOW_ENV') {
                    continue
                }
                # Strip optional surrounding single quotes from the value
                $value = ($Matches[2] -replace "^'", '') -replace "'$", ''
                $priorValues[$name] = [System.Environment]::GetEnvironmentVariable($name)
                [System.Environment]::SetEnvironmentVariable($name, $value)
                Write-Verbose "Set $name=$value"
            }
        }

        Write-Verbose -Verbose "Set $($priorValues.Count) cargo-llvm-cov environment variables"
        $priorValues
    }
}

function Reset-LlvmCovEnvironment {
    <#
        .SYNOPSIS
        Restores environment variables modified by Set-LlvmCovEnvironment.

        .PARAMETER PriorValues
        The hashtable returned by Set-LlvmCovEnvironment containing original values.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [hashtable]$PriorValues
    )

    process {
        foreach ($entry in $PriorValues.GetEnumerator()) {
            [System.Environment]::SetEnvironmentVariable($entry.Key, $entry.Value)
        }
        Write-Verbose -Verbose "Restored $($PriorValues.Count) environment variables"
    }
}

function Export-CodeCoverageReport {
    <#
        .SYNOPSIS
        Generates an LCOV code coverage report from collected profile data.

        .DESCRIPTION
        Runs `cargo llvm-cov report` to produce an LCOV-formatted coverage report from the
        profile data collected during an instrumented build and test run.

        .PARAMETER OutputPath
        The file path where the LCOV report will be written.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$OutputPath
    )

    process {
        Write-Verbose -Verbose "Generating LCOV report at: $OutputPath"
        cargo llvm-cov report --lcov --output-path $OutputPath
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to generate code coverage report at '$OutputPath'"
        }
        Write-Verbose -Verbose "Code coverage report written to: $OutputPath"
    }
}

function Export-PesterCodeCoverageReport {
    <#
        .SYNOPSIS
        Generates an LCOV code coverage report from profraw files produced by instrumented binaries.

        .DESCRIPTION
        Uses llvm-profdata and llvm-cov directly (from the rustup llvm-tools-preview component)
        to merge raw profile data and export an LCOV report. This is used by Pester test jobs
        that run instrumented binaries outside of the cargo build environment.

        .PARAMETER BinDirectory
        Path to the directory containing instrumented binaries (e.g., bin/).

        .PARAMETER ProfileDirectory
        Path to the directory containing .profraw files produced by instrumented binaries.

        .PARAMETER OutputPath
        The file path where the LCOV report will be written.

        .PARAMETER SourceDirectory
        Optional path to the source directory for source-level mapping. Defaults to the
        repository root.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$BinDirectory,

        [Parameter(Mandatory)]
        [string]$ProfileDirectory,

        [Parameter(Mandatory)]
        [string]$OutputPath,

        [Parameter()]
        [string]$SourceDirectory = $PSScriptRoot
    )

    process {
        # Find the llvm tools from rustup
        $toolchainPath = & rustc --print sysroot 2>$null
        if (-not $toolchainPath) {
            throw 'Could not determine Rust toolchain sysroot. Ensure rustc is installed.'
        }

        $llvmBinDir = Join-Path $toolchainPath 'lib' 'rustlib' (& rustc -vV |
            Select-String 'host: (.+)' | ForEach-Object { $_.Matches[0].Groups[1].Value }) 'bin'

        $llvmProfdata = Join-Path $llvmBinDir 'llvm-profdata'
        $llvmCov = Join-Path $llvmBinDir 'llvm-cov'

        if ($IsWindows) {
            $llvmProfdata += '.exe'
            $llvmCov += '.exe'
        }

        if (-not (Test-Path $llvmProfdata)) {
            throw "llvm-profdata not found at '$llvmProfdata'. Ensure llvm-tools-preview is installed via: rustup component add llvm-tools-preview"
        }

        # Find all profraw files
        $profrawFiles = Get-ChildItem -Path $ProfileDirectory -Filter '*.profraw' -Recurse
        if ($profrawFiles.Count -eq 0) {
            Write-Warning "No .profraw files found in '$ProfileDirectory'. Coverage report will be empty."
            return
        }
        Write-Verbose -Verbose "Found $($profrawFiles.Count) profraw file(s)"

        # Merge profraw files into a single profdata file
        $profdataPath = Join-Path $ProfileDirectory 'merged.profdata'
        $mergeArgs = @('merge', '-sparse')
        $mergeArgs += $profrawFiles.FullName
        $mergeArgs += @('-o', $profdataPath)

        Write-Verbose -Verbose "Merging profraw files into: $profdataPath"
        & $llvmProfdata @mergeArgs
        if ($LASTEXITCODE -ne 0) {
            throw "llvm-profdata merge failed with exit code $LASTEXITCODE"
        }

        # Find all executable binaries in the bin directory
        $binaries = if ($IsWindows) {
            Get-ChildItem -Path $BinDirectory -Filter '*.exe' -File
        } else {
            Get-ChildItem -Path $BinDirectory -File | Where-Object {
                # On Unix, check if file is executable
                (& test -x $_.FullName) -and $_.Extension -notin @('.pdb', '.d', '.ps1', '.psm1', '.psd1', '.json', '.yaml', '.yml', '.txt', '.md')
            }
        }

        if ($binaries.Count -eq 0) {
            Write-Warning "No executable binaries found in '$BinDirectory'. Cannot generate coverage report."
            return
        }
        Write-Verbose -Verbose "Using $($binaries.Count) binary file(s) for coverage export"

        # Build llvm-cov export arguments
        # First binary is the primary, additional are specified via -object
        $covArgs = @(
            'export'
            '-format=lcov'
            "-instr-profile=$profdataPath"
            '--ignore-filename-regex=\.cargo|rustc'
        )

        $covArgs += $binaries[0].FullName
        for ($i = 1; $i -lt $binaries.Count; $i++) {
            $covArgs += @('-object', $binaries[$i].FullName)
        }

        Write-Verbose -Verbose "Exporting LCOV report to: $OutputPath"
        & $llvmCov @covArgs > $OutputPath 2>$null
        if ($LASTEXITCODE -ne 0) {
            Write-Warning "llvm-cov export returned exit code $LASTEXITCODE. Report may be incomplete."
        }

        if (Test-Path $OutputPath) {
            $fileSize = (Get-Item $OutputPath).Length
            Write-Verbose -Verbose "Code coverage report written to: $OutputPath ($fileSize bytes)"
        } else {
            Write-Warning "Coverage report was not generated at: $OutputPath"
        }
    }
}

function Merge-LcovFile {
    <#
        .SYNOPSIS
        Merges multiple LCOV files into a single consolidated report.

        .DESCRIPTION
        Reads multiple LCOV-format coverage files and merges them by combining line hit
        counts for matching source files. When the same line appears in multiple reports,
        the hit counts are summed.

        .PARAMETER Path
        Array of paths to LCOV files to merge.

        .PARAMETER OutputPath
        The file path where the merged LCOV report will be written.
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string[]]$Path,

        [Parameter(Mandatory)]
        [string]$OutputPath
    )

    process {
        # Structure: $coverage[sourceFile][lineNumber] = hitCount
        $coverage = @{}
        $functionData = @{}

        foreach ($lcovPath in $Path) {
            if (-not (Test-Path $lcovPath)) {
                Write-Verbose "Skipping missing LCOV file: $lcovPath"
                continue
            }

            $currentFile = $null
            foreach ($line in Get-Content -Path $lcovPath) {
                if ($line -match '^SF:(.+)$') {
                    $currentFile = $Matches[1]
                    if (-not $coverage.ContainsKey($currentFile)) {
                        $coverage[$currentFile] = @{}
                        $functionData[$currentFile] = [System.Collections.Generic.List[string]]::new()
                    }
                } elseif ($line -match '^DA:(\d+),(\d+)') {
                    $lineNum = [int]$Matches[1]
                    $hits = [int]$Matches[2]
                    if ($currentFile -and $coverage.ContainsKey($currentFile)) {
                        if ($coverage[$currentFile].ContainsKey($lineNum)) {
                            $coverage[$currentFile][$lineNum] += $hits
                        } else {
                            $coverage[$currentFile][$lineNum] = $hits
                        }
                    }
                } elseif ($line -match '^(FN|FNDA|FNF|FNH):' -and $currentFile) {
                    $functionData[$currentFile].Add($line)
                }
            }
        }

        # Write merged output
        $output = [System.Text.StringBuilder]::new()
        foreach ($file in $coverage.Keys | Sort-Object) {
            [void]$output.AppendLine("SF:$file")
            foreach ($fn in $functionData[$file]) {
                [void]$output.AppendLine($fn)
            }
            $lineCount = 0
            $hitCount = 0
            foreach ($lineNum in $coverage[$file].Keys | Sort-Object) {
                $hits = $coverage[$file][$lineNum]
                [void]$output.AppendLine("DA:$lineNum,$hits")
                $lineCount++
                if ($hits -gt 0) { $hitCount++ }
            }
            [void]$output.AppendLine("LF:$lineCount")
            [void]$output.AppendLine("LH:$hitCount")
            [void]$output.AppendLine('end_of_record')
        }

        Set-Content -Path $OutputPath -Value $output.ToString() -NoNewline
        Write-Verbose -Verbose "Merged $($Path.Count) LCOV file(s) into: $OutputPath"
    }
}

function Show-CodeCoverageReport {
    <#
        .SYNOPSIS
        Displays a colorized visualization of code coverage on changed lines.

        .DESCRIPTION
        Reads source files and displays changed executable lines with green for covered
        and red with underline for uncovered, using $PSStyle for ANSI formatting.

        .PARAMETER FileDetails
        Array of objects with File (path) and LineCoverageMap (hashtable of line number to bool).
    #>
    [CmdletBinding()]
    param(
        [Parameter()]
        [AllowEmptyCollection()]
        [PSCustomObject[]]$FileDetails = @()
    )

    process {
        if ($FileDetails.Count -eq 0) {
            return
        }
        foreach ($detail in $FileDetails) {
            $filePath = $detail.File
            $lineCoverageMap = $detail.LineCoverageMap

            if ($lineCoverageMap.Count -eq 0) {
                continue
            }

            $fileContent = Get-Content -Path $filePath -ErrorAction SilentlyContinue
            if (-not $fileContent) {
                continue
            }

            Write-Host ""
            Write-Host "$($PSStyle.Bold)$filePath$($PSStyle.BoldOff)" -ForegroundColor Cyan

            $sortedLines = $lineCoverageMap.Keys | Sort-Object
            $lineNumWidth = ($sortedLines[-1]).ToString().Length

            foreach ($lineNum in $sortedLines) {
                $lineIndex = $lineNum - 1
                $lineText = if ($lineIndex -lt $fileContent.Count) { $fileContent[$lineIndex] } else { '' }
                $prefix = $lineNum.ToString().PadLeft($lineNumWidth)

                if ($lineCoverageMap[$lineNum]) {
                    # Covered - green
                    Write-Host "$($PSStyle.Foreground.Green)  $prefix | $lineText$($PSStyle.Reset)"
                } else {
                    # Uncovered - red with underline
                    Write-Host "$($PSStyle.Foreground.Red)$($PSStyle.Underline)  $prefix | $lineText$($PSStyle.UnderlineOff)$($PSStyle.Reset)"
                }
            }
        }
        Write-Host ""
    }
}

function Get-CodeCoverageReport {
    <#
        .SYNOPSIS
        Analyzes code coverage on changed files and returns a coverage report object.

        .DESCRIPTION
        Parses an LCOV file and cross-references it with git diff output to determine what
        percentage of changed executable lines are covered by tests.

        .PARAMETER LcovPath
        Path to the LCOV coverage report file.

        .PARAMETER BaseSha
        The base commit SHA to compare from.

        .PARAMETER HeadSha
        The head commit SHA to compare to.

        .OUTPUTS
        PSCustomObject with properties: Percentage, CoveredLines, TotalLines, Emoji, Label
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$LcovPath,

        [Parameter(Mandatory)]
        [string]$BaseSha,

        [Parameter(Mandatory)]
        [string]$HeadSha
    )

    process {
        if (-not (Test-Path $LcovPath)) {
            throw "LCOV file not found at '$LcovPath'"
        }

        # Parse the LCOV file into a hashtable keyed by source file path
        $lcovData = @{}
        $currentFile = $null
        foreach ($line in Get-Content -Path $LcovPath) {
            if ($line -match '^SF:(.+)$') {
                $currentFile = $Matches[1]
                $lcovData[$currentFile] = @{}
            } elseif ($line -match '^DA:(\d+),(\d+)' -and $currentFile) {
                $lineNum = [int]$Matches[1]
                # LLVM emits sentinel values near UInt64.MaxValue for uninstrumented lines
                $rawHit = [decimal]$Matches[2]
                $hitCount = if ($rawHit -gt [long]::MaxValue) { 0 } else { [long]$rawHit }
                $lcovData[$currentFile][$lineNum] = $hitCount
            } elseif ($line -eq 'end_of_record') {
                $currentFile = $null
            }
        }

        # Get changed Rust files
        $changedFiles = Get-ChangedRustFile -BaseSha $BaseSha -HeadSha $HeadSha

        $totalChangedLines = 0
        $coveredLines = 0
        # Collect per-file coverage detail for visualization
        $fileDetails = @()

        foreach ($file in $changedFiles) {
            if (-not $file -or -not (Test-Path $file)) {
                continue
            }

            # Parse diff to get added line numbers in the new file
            $diffOutput = git diff "$BaseSha..$HeadSha" -- $file
            $addedLineNumbers = @()
            $currentLineNum = 0

            foreach ($diffLine in $diffOutput) {
                if ($diffLine -match '^@@\s+\-\d+(?:,\d+)?\s+\+(\d+)(?:,\d+)?\s+@@') {
                    $currentLineNum = [int]$Matches[1]
                } elseif ($diffLine.StartsWith('+') -and -not $diffLine.StartsWith('+++')) {
                    $addedLineNumbers += $currentLineNum
                    $currentLineNum++
                } elseif ($diffLine.StartsWith('-') -and -not $diffLine.StartsWith('---')) {
                    # Deleted lines don't advance the new file line counter
                } else {
                    $currentLineNum++
                }
            }

            # Find matching LCOV entry for this file
            $absPath = (Resolve-Path $file).Path
            $normalizedFile = $file.Replace('\', '/')
            $fileCoverage = $null
            foreach ($key in $lcovData.Keys) {
                $normalizedKey = $key.Replace('\', '/')
                if ($normalizedKey -eq $absPath -or
                    $normalizedKey -eq $normalizedFile -or
                    $normalizedKey.EndsWith("/$normalizedFile") -or
                    $normalizedKey.EndsWith("\$normalizedFile")) {
                    $fileCoverage = $lcovData[$key]
                    break
                }
            }

            if (-not $fileCoverage) {
                Write-Verbose -Verbose "No LCOV match for '$file' (absPath='$absPath'). LCOV keys: $($lcovData.Keys -join ', ')"
            }

            # Build per-line coverage map for this file (only added executable lines)
            $lineCoverageMap = @{}
            if ($fileCoverage) {
                foreach ($lineNum in $addedLineNumbers) {
                    if ($fileCoverage.ContainsKey($lineNum)) {
                        $totalChangedLines++
                        $isCovered = $fileCoverage[$lineNum] -gt 0
                        if ($isCovered) {
                            $coveredLines++
                        }
                        $lineCoverageMap[$lineNum] = $isCovered
                    }
                }
            } else {
                # File not in coverage report - count added lines as uncovered
                $totalChangedLines += $addedLineNumbers.Count
                foreach ($lineNum in $addedLineNumbers) {
                    $lineCoverageMap[$lineNum] = $false
                }
            }

            if ($lineCoverageMap.Count -gt 0) {
                $fileDetails += [PSCustomObject]@{
                    File            = $file
                    LineCoverageMap = $lineCoverageMap
                }
            }
        }

        if ($totalChangedLines -eq 0) {
            $percentage = 100
        } else {
            $percentage = [int][math]::Floor($coveredLines * 100 / $totalChangedLines)
        }

        # Determine emoji and label
        $emoji, $label = if ($percentage -eq 100) {
            '😲', '100% coverage'
        } elseif ($percentage -ge 90) {
            '😁', '90%+ coverage'
        } elseif ($percentage -ge 80) {
            '😊', '80%+ coverage'
        } elseif ($percentage -ge 70) {
            '😐', '70%+ coverage'
        } else {
            '😢', 'less than 70% coverage'
        }

        Write-Verbose -Verbose "Coverage: $percentage% ($coveredLines/$totalChangedLines executable lines covered)"

        # Show visual report
        Show-CodeCoverageReport -FileDetails $fileDetails

        [PSCustomObject]@{
            Percentage   = $percentage
            CoveredLines = $coveredLines
            TotalLines   = $totalChangedLines
            Emoji        = $emoji
            Label        = $label
        }
    }
}
function Get-FullCodeCoverageReport {
    <#
        .SYNOPSIS
        Computes overall code coverage statistics from an LCOV file across the entire codebase.

        .DESCRIPTION
        Parses an LCOV file and computes total line coverage across all source files,
        providing a full-codebase coverage percentage regardless of what changed in a PR.

        .PARAMETER LcovPath
        Path to the LCOV coverage report file.

        .OUTPUTS
        PSCustomObject with properties: Percentage, CoveredLines, TotalLines, Emoji, Label
    #>
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [string]$LcovPath
    )

    process {
        if (-not (Test-Path $LcovPath)) {
            throw "LCOV file not found at '$LcovPath'"
        }

        $totalLines = 0
        $coveredLines = 0

        foreach ($line in Get-Content -Path $LcovPath) {
            if ($line -match '^DA:(\d+),(\d+)') {
                $rawHit = [decimal]$Matches[2]
                # LLVM emits sentinel values near UInt64.MaxValue for uninstrumented lines
                $hitCount = if ($rawHit -gt [long]::MaxValue) { 0 } else { [long]$rawHit }
                $totalLines++
                if ($hitCount -gt 0) {
                    $coveredLines++
                }
            }
        }

        if ($totalLines -eq 0) {
            $percentage = 0
        } else {
            $percentage = [int][math]::Floor($coveredLines * 100 / $totalLines)
        }

        $emoji, $label = if ($percentage -ge 90) {
            ':green_circle:', 'excellent'
        } elseif ($percentage -ge 80) {
            ':large_blue_circle:', 'good'
        } elseif ($percentage -ge 70) {
            ':yellow_circle:', 'acceptable'
        } elseif ($percentage -ge 60) {
            ':orange_circle:', 'needs improvement'
        } else {
            ':red_circle:', 'low'
        }

        Write-Verbose -Verbose "Full codebase coverage: $percentage% ($coveredLines/$totalLines lines)"

        [PSCustomObject]@{
            Percentage   = $percentage
            CoveredLines = $coveredLines
            TotalLines   = $totalLines
            Emoji        = $emoji
            Label        = $label
        }
    }
}

#endregion Code coverage functions

#region    Test project functions
function Test-RustProject {
    [CmdletBinding()]
    param(
        [DscProjectDefinition[]]$Project,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$Docs,
        [string]$TestFilter
    )

    begin {
        $flags = @($Release ? '-r' : $null)
        if ($Architecture -ne 'current') {
            $flags += '--target'
            $flags += $Architecture
        } else {
            $memberGroup = if ($IsLinux) {
                'Linux'
            } elseif ($IsMacOS) {
                'macOS'
            } elseif ($IsWindows) {
                'Windows'
            }
            Set-DefaultWorkspaceMemberGroup -MemberGroup $memberGroup
        }
        if ($Docs) {
            $flags += '--doc'
        }
    }

    process {
        $members = Get-DefaultWorkspaceMemberGroup
        if ($Docs) {
            Write-Verbose -Verbose "Testing documentation for rust projects: [$members]"
        } else {
            Write-Verbose -Verbose "Testing rust projects: [$members]"
        }
        if (-not [string]::IsNullOrEmpty($TestFilter)) {
            cargo test @flags -- $TestFilter
        } else {
            cargo test @flags
        }

        if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
            throw "Last exit code is $LASTEXITCODE, rust tests failed"
        }
    }

    clean {
        Reset-DefaultWorkspaceMemberGroup
    }
}

function Test-ProjectWithPester {
    [cmdletbinding()]
    param(
        [DscProjectDefinition[]]$Project,
        [ValidateSet("dsc", "adapters", "extensions", "grammars", "resources")]
        [string[]]$Group,
        [switch]$UsingADO
    )

    begin {
        Write-verbose "PSModulePath is:`n`t$($env:PSModulePath)"
        Write-Verbose "Pester module located in:`n`t$((Get-Module -Name Pester -ListAvailable).Path)"
        $repository = $UsingADO ? 'CFS' : 'PSGallery'

        if ($IsWindows) {
            Write-Verbose "Disabling duplicated WinPS resources that break PSDesiredStateConfiguration module"
            $a = $env:PSModulePath -split ";" | Where-Object { $_ -notmatch 'WindowsPowerShell' }
            $env:PSModulePath = $a -join ';'

            Write-Verbose "Updated PSModulePath is:`n`t$($env:PSModulePath)"
            if (-not (Get-Module -ListAvailable -Name Pester)) {
                Write-Verbose "Installing module Pester to execute tests from $repository..."
                $InstallTargetDir = ($env:PSModulePath -split ";")[0]
                Find-PSResource -Name 'Pester' -Repository $repository
                | Save-PSResource -Path $InstallTargetDir -TrustRepository
            }

            Write-Verbose "Updated Pester module location:`n`t$((Get-Module -Name Pester -ListAvailable).Path)"
        }
        $pesterParams = @{
            Output = 'Detailed'
            ErrorAction = 'Stop'
        }
        if ($Project) {
            $pesterParams.ExcludePath = $Project.RelativePath | Where-Object -FilterScript {
                $_.Name -notin $Project.Name
            }
        }
        if ($Group) {
            $pesterParams.Path = $Group
        }
    }

    process {
        if ($Group -and $Project) {
            Write-Verbose (@(
                "Invoking pester for groups and projects:"
                "Groups: [$($Group -join ', ')]"
                "Projects: [$($Project.Name -join ', ')]"
            ) -join "`n`t")
        } elseif ($Group) {
            Write-Verbose "Invoking pester for groups: [$($Group -join ', ')]"
        } elseif ($Project) {
            Write-Verbose "Invoking pester for projects: [$($Project.Name -join ', ')]"
        } else {
            Write-Verbose "Invoking pester for all groups and projects"
        }
        Invoke-Pester @pesterParams
    }
}
#endregion Test project functions

#region Package project functions
function Build-DscDebPackage{
    [CmdletBinding()]
    param(
        [DscProjectBuildData]$BuildData,
        [DscArtifactDirectoryPath]$ArtifactDirectory,
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $Architecture = 'current',
        [switch]$Release
    )

    begin {
        Write-Verbose -Verbose "Starting DEB package creation for architecture '$Architecture'"
        if (!$IsLinux) {
            throw "DEB package creation is only supported on Linux"
        }

        # Check if dpkg-deb is available
        if ($null -eq (Get-Command dpkg-deb -ErrorAction Ignore)) {
            throw "dpkg-deb not found. Please install dpkg package (e.g., 'sudo apt install dpkg' or 'sudo dnf install dpkg')"
        }

        if ($null -eq $BuildData) {
            $BuildData = Import-DscBuildData
        }
        $filesForPackage = $BuildData.PackageFiles.Linux
        if ($null -eq $ArtifactDirectory) {
            $artifactDirectory   = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        }

        $debTarget = $artifactDirectory.DebTarget
        $bin = $artifactDirectory.Bin
        $productVersion = Get-DscCliVersion
        # Determine DEB architecture
        $debArch = if ($architecture -eq 'current') {
            # Detect current system architecture
            $currentArch = uname -m
            if ($currentArch -eq 'x86_64') {
                'amd64'
            } elseif ($currentArch -eq 'aarch64') {
                'arm64'
            } else {
                throw "Unsupported current architecture for DEB: $currentArch"
            }
        } elseif ($architecture -eq 'aarch64-unknown-linux-musl' -or $architecture -eq 'aarch64-unknown-linux-gnu') {
            'arm64'
        } elseif ($architecture -eq 'x86_64-unknown-linux-musl' -or $architecture -eq 'x86_64-unknown-linux-gnu') {
            'amd64'
        } else {
            throw "Unsupported architecture for DEB: $architecture"
        }

        Write-Verbose -Verbose "Building DEB package"
        $debPackageName = "dsc_$productVersion-1_$debArch.deb"
        $finalDebPath = Join-Path $artifactDirectory.BinRoot $debPackageName
    }

    process {
        if (Test-Path $debTarget) {
            Remove-Item $debTarget -Recurse -ErrorAction Stop -Force
        }

        New-Item -ItemType Directory $debTarget > $null

        # Create DEB package structure
        $debBuildRoot = Join-Path $debTarget 'dsc'
        $debDirs = @('DEBIAN', 'opt/dsc', 'usr/bin')
        foreach ($dir in $debDirs) {
            New-Item -ItemType Directory -Path (Join-Path $debBuildRoot $dir) -Force > $null
        }

        $stagingDir = Join-Path $debBuildRoot 'opt' 'dsc'

        foreach ($file in $filesForPackage) {
            if ((Get-Item "$bin\$file") -is [System.IO.DirectoryInfo]) {
                Copy-Item "$bin\$file" "$stagingDir\$file" -Recurse -ErrorAction Stop
            } else {
                Copy-Item "$bin\$file" $stagingDir -ErrorAction Stop
            }
        }

        # Create symlinks in usr/bin
        $symlinkPath = Join-Path $debBuildRoot 'usr' 'bin' 'dsc'
        New-Item -ItemType SymbolicLink -Path $symlinkPath -Target '/opt/dsc/dsc' -Force > $null

        $symlinkPath = Join-Path $debBuildRoot 'usr' 'bin' 'dsc-bicep-ext'
        New-Item -ItemType SymbolicLink -Path $symlinkPath -Target '/opt/dsc/dsc-bicep-ext' -Force > $null

        # Read the control template and replace placeholders
        $controlTemplate = Get-Content "$PSScriptRoot/packaging/deb/control" -Raw
        $controlContent = $controlTemplate.Replace('VERSION_PLACEHOLDER', $productVersion).Replace('ARCH_PLACEHOLDER', $debArch)
        $controlFile = Join-Path $debBuildRoot 'DEBIAN' 'control'
        Set-Content -Path $controlFile -Value $controlContent

        # Build the DEB
        dpkg-deb --build $debBuildRoot 2>&1 > $debTarget/debbuild.log

        if ($LASTEXITCODE -ne 0) {
            Write-Error (Get-Content $debTarget/debbuild.log -Raw)
            throw "Failed to create DEB package"
        }

        # Move the DEB to the bin directory with the correct name
        $builtDeb = "$debBuildRoot.deb"
        if (!(Test-Path $builtDeb)) {
            throw "DEB package was not created"
        }

        Move-Item $builtDeb $finalDebPath -Force
        Write-Host -ForegroundColor Green "`nDEB package is created at $finalDebPath"
    }
}

function Build-DscMsixPackage {
    [CmdletBinding()]
    param(
        [DscProjectBuildData]$BuildData,
        [DscArtifactDirectoryPath]$ArtifactDirectory,
        [ValidateSet(
            'msix',
            'msix-private',
            'msixbundle'
        )]
        $packageType,
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$UseX64MakeAppx
    )

    begin {
        Write-Verbose -Verbose "Starting MSIX package creation for architecture '$Architecture' and package type '$packageType'"
        if (!$IsWindows) {
            throw "MSIX packaging is only supported on Windows"
        }
        if ($null -eq $BuildData) {
            $BuildData = Import-DscBuildData
        }
        if ($null -eq $ArtifactDirectory) {
            $artifactDirectory   = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        }

        $productVersion = Get-DscCliVersion
        $isPrivate = $packageType -eq 'msix-private'
        $isPreview = $productVersion -like '*-*'
        $makeappx = Find-MakeAppx -UseX64MakeAppx:$UseX64MakeAppx
        $makepri  = Get-Item (Join-Path $makeappx.Directory "makepri.exe") -ErrorAction Stop
    }

    process {
        if ($packageType -eq 'msixbundle') {
            $packageName = "DSC-$productVersion-Win"
            $msixArguments = @(
                'bundle'
                '/d', $artifactDirectory.MsixBundle
                '/p', "$($artifactDirectory.BinRoot)\$packageName.msixbundle"
            )
            & $makeappx @msixArguments
            return
        }

        if ($architecture -eq 'current') {
            throw 'MSIX requires a specific architecture'
        }

        $displayName = 'Desired State Configuration'
        $productName = 'DesiredStateConfiguration'

        if ($isPreview) {
            Write-Verbose -Verbose "Preview version detected: $productVersion"
            if ($isPrivate) {
                $productName += "-Private"
            }
            else {
                $productName += "-Preview"
            }

            # save preview number
            $previewNumber = [int]($productVersion -replace '.*?-[a-z]+\.([0-9]+)', '$1' | Out-String)
            $productLabel = $productVersion.Split('-')[1]
            if ($productLabel.StartsWith('rc')) {
                # if RC, we increment by 100 to ensure it's newer than the last preview
                $previewNumber += 100
            }
            # remove label from version
            $productVersion = $productVersion.Split('-')[0]
            # replace revision number with preview number
            $productVersion = $productVersion -replace '(\d+)$', "$previewNumber.0"

            if ($isPrivate) {
                $displayName += " (Private)"
            }
            else {
                $displayName += " (Preview)"
            }
        } else {
            # appx requires a version in the format of major.minor.build.revision with revision being 0
            $productVersion += ".0"
        }

        Write-Verbose -Verbose "Product version is $productVersion"
        $arch = ($architecture -eq 'aarch64-pc-windows-msvc') ? 'arm64' : 'x64'

        # Appx manifest needs to be in root of source path, but the embedded version needs to be updated
        # Retrieve manifest and set version correctly
        $appxManifest = Get-Content "$PSScriptRoot\packaging\msix\AppxManifest.xml" -Raw
        if ($Release) {
            # CP-459155 is 'CN=Microsoft Windows Store Publisher (Store EKU), O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
            # authenticodeFormer is 'CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
            $publisher = 'CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
        } else {
            # For debug builds, use a self-signed developer identity per
            # https://learn.microsoft.com/en-us/windows/msix/package/unsigned-package
            $publisher = 'CN=AppModelSamples, OID.2.25.311729368913984317654407730594956997722=1'
        }
        $appxManifest = $appxManifest.Replace('$VERSION$', $ProductVersion).Replace('$ARCH$', $Arch).Replace('$PRODUCTNAME$', $productName).Replace('$DISPLAYNAME$', $displayName).Replace('$PUBLISHER$', $publisher)
        # Remove the output directory if it already exists, then recreate it.
        $msixTarget = $artifactDirectory.MsixTarget
        if (Test-Path $msixTarget) {
            Remove-Item $msixTarget -Recurse -ErrorAction Stop -Force
        }
        New-Item -ItemType Directory $msixTarget -Force > $null
        # Copy the manifest contents into the output directory.
        Set-Content -Path "$msixTarget\AppxManifest.xml" -Value $appxManifest -Force

        $bin = $artifactDirectory.Bin
        foreach ($file in $BuildData.PackageFiles.Windows) {
            if ((Get-Item "$bin\$file") -is [System.IO.DirectoryInfo]) {
                Copy-Item "$bin\$file" "$msixTarget\$file" -Recurse -ErrorAction Stop
            } else {
                Copy-Item "$bin\$file" $msixTarget -ErrorAction Stop
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
        $makepriArguments = @(
            'createconfig'
            '/o'
            '/cf', (Join-Path $msixTarget "priconfig.xml")
            '/dq', 'en-US'
        )
        & $makepri @makepriArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create priconfig.xml"
        }

        Write-Verbose "Creating resources.pri" -Verbose
        Push-Location $msixTarget
        $makepriArguments = @(
            'new'
            '/v'
            '/o'
            '/pr', $msixTarget
            '/cf', (Join-Path $msixTarget "priconfig.xml")
        )
        & $makepri @makepriArguments
        Pop-Location
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create resources.pri"
        }

        Write-Verbose "Creating msix package" -Verbose

        $targetFolder = $artifactDirectory.MsixBundle
        if (Test-Path $targetFolder) {
            Remove-Item $targetFolder -Recurse -ErrorAction Stop -Force
        }
        New-Item -ItemType Directory -Path $targetFolder -Force > $null

        $packageName = Join-Path $targetFolder "$productName-$productVersion-$arch.msix"
        $makeappxArguments = @(
            'pack'
            '/o'
            '/v'
            '/h', 'SHA256'
            '/d', $msixTarget
            '/p', $packageName
        )
        & $makeappx @makeappxArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create msix package"
        }

        if ($Release) {
            Write-Host -ForegroundColor Green "`nMSIX package is created at $packageName"
        } else {
            Write-Host -ForegroundColor Green "`nInstall the debug MSIX package with:`nAdd-AppxPackage -AllowUnsigned -Path $packageName"
        }
    }
}

function Build-DscRpmPackage {
    [CmdletBinding()]
    param(
        [DscProjectBuildData]$BuildData,
        [DscArtifactDirectoryPath]$ArtifactDirectory,
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $Architecture = 'current',
        [switch]$Release
    )

    begin {
        Write-Verbose -Verbose "Starting RPM package creation for architecture '$Architecture'"
        if (!$IsLinux) {
            throw "RPM package creation is only supported on Linux"
        }

        # Check if rpmbuild is available
        if ($null -eq (Get-Command rpmbuild -ErrorAction Ignore)) {
            throw "rpmbuild not found. Please install rpm-build package (e.g., 'sudo apt install rpm build-essential' or 'sudo dnf install rpm-build')"
        }

        if ($null -eq $BuildData) {
            $BuildData = Import-DscBuildData
        }
        $filesForPackage = $BuildData.PackageFiles.Linux
        if ($null -eq $ArtifactDirectory) {
            $artifactDirectory   = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        }

        $rpmTarget = $artifactDirectory.RpmTarget
        $bin = $artifactDirectory.Bin
        $productVersion = Get-DscCliVersion
        # Determine RPM architecture
        $rpmArch = if ($architecture -eq 'current') {
            # Detect current system architecture
            $currentArch = uname -m
            if ($currentArch -eq 'x86_64') {
                'x86_64'
            } elseif ($currentArch -eq 'aarch64') {
                'aarch64'
            } else {
                throw "Unsupported current architecture for RPM: $currentArch"
            }
        } elseif ($architecture -eq 'aarch64-unknown-linux-musl' -or $architecture -eq 'aarch64-unknown-linux-gnu') {
            'aarch64'
        } elseif ($architecture -eq 'x86_64-unknown-linux-musl' -or $architecture -eq 'x86_64-unknown-linux-gnu') {
            'x86_64'
        } else {
            throw "Unsupported architecture for RPM: $architecture"
        }

        Write-Verbose -Verbose "Building RPM package"
        $rpmPackageName = "dsc_$productVersion-1_$rpmArch.rpm"
        $finalRpmPath = Join-Path $artifactDirectory.BinRoot $rpmPackageName
    }

    process {
        if (Test-Path $rpmTarget) {
            Remove-Item $rpmTarget -Recurse -ErrorAction Stop -Force
        }

        New-Item -ItemType Directory $rpmTarget > $null

        # Create RPM build directories
        $rpmBuildRoot = Join-Path $rpmTarget 'rpmbuild'
        $rpmDirs = @('BUILD', 'RPMS', 'SOURCES', 'SPECS', 'SRPMS')
        foreach ($dir in $rpmDirs) {
            New-Item -ItemType Directory -Path (Join-Path $rpmBuildRoot $dir) -Force > $null
        }

        # Create a staging directory for the files
        $stagingDir = Join-Path $rpmBuildRoot 'SOURCES' 'dsc_files'
        New-Item -ItemType Directory $stagingDir > $null

        foreach ($file in $filesForPackage) {
            if ((Get-Item "$bin\$file") -is [System.IO.DirectoryInfo]) {
                Copy-Item "$bin\$file" "$stagingDir\$file" -Recurse -ErrorAction Stop
            } else {
                Copy-Item "$bin\$file" $stagingDir -ErrorAction Stop
            }
        }

        # Read the spec template and replace placeholders
        $specTemplate = Get-Content "$PSScriptRoot/packaging/rpm/dsc.spec" -Raw
        $specContent = $specTemplate.Replace('VERSION_PLACEHOLDER', $productVersion.Replace('-','~')).Replace('ARCH_PLACEHOLDER', $rpmArch)
        $specFile = Join-Path $rpmBuildRoot 'SPECS' 'dsc.spec'
        Set-Content -Path $specFile -Value $specContent

        Write-Verbose -Verbose "Building RPM package"
        # Build the RPM
        rpmbuild -v -bb --define "_topdir $rpmBuildRoot" --buildroot "$rpmBuildRoot/BUILDROOT" $specFile 2>&1 > $rpmTarget/rpmbuild.log

        if ($LASTEXITCODE -ne 0) {
            Write-Error (Get-Content $rpmTarget/rpmbuild.log -Raw)
            throw "Failed to create RPM package"
        }

        # Copy the RPM to the bin directory
        $builtRpm = Get-ChildItem -Path (Join-Path $rpmBuildRoot 'RPMS') -Recurse -Filter '*.rpm' | Select-Object -First 1
        if ($null -eq $builtRpm) {
            throw "RPM package was not created"
        }

        $finalRpmPath = Join-Path $PSScriptRoot 'bin' $builtRpm.Name
        Copy-Item $builtRpm.FullName $finalRpmPath -Force

        Write-Host -ForegroundColor Green "`nRPM package is created at $finalRpmPath"
    }
}

function Build-DscZipPackage {
    [CmdletBinding()]
    param(
        [DscProjectBuildData]$BuildData,
        [DscArtifactDirectoryPath]$ArtifactDirectory,
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $Architecture = 'current',
        [switch]$Release
    )

    begin {
        Write-Verbose -Verbose "Starting ZIP package creation for architecture '$Architecture'"
        if ($Architecture -eq 'current') {
            throw 'Building a zip package requires a specific architecture targeting Windows'
        }
        if ($Architecture -notmatch 'windows') {
            throw "Invalid architecture '$Architecture' - can only build zip package for Windows target"
        }

        if ($null -eq $BuildData) {
            $BuildData = Import-DscBuildData
        }
        if ($null -eq $ArtifactDirectory) {
            $artifactDirectory   = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        }

        $productVersion    = Get-DscCliVersion
        $packageName       = "DSC-$productVersion-$architecture.zip"
        $artifactDirectory = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        $zipTarget         = $artifactDirectory.ZipTarget
        $bin               = $artifactDirectory.Bin
        $zipFile           = Join-Path $artifactDirectory.BinRoot $packageName
    }

    process {
        Write-Verbose "Building zip package for architecture '$Architecture'"
        if (Test-Path $zipTarget) {
            Remove-Item $zipTarget -Recurse -ErrorAction Stop -Force
        }
        New-Item -ItemType Directory $zipTarget > $null

        foreach ($file in $BuildData.PackageFiles.Windows) {
            if ((Get-Item "$bin\$file") -is [System.IO.DirectoryInfo]) {
                Copy-Item "$bin\$file" "$zipTarget\$file" -Recurse -ErrorAction Stop
            } else {
                Copy-Item "$bin\$file" $zipTarget -ErrorAction Stop
            }
        }

        Compress-Archive -Path "$zipTarget/*" -DestinationPath $zipFile -Force
        Write-Host -ForegroundColor Green "`nZip file is created at $zipFile"
    }
}

function Build-DscTgzPackage {
    [CmdletBinding()]
    param(
        [DscProjectBuildData]$BuildData,
        [DscArtifactDirectoryPath]$artifactDirectory,
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $Architecture = 'current',
        [switch]$Release
    )

    begin {
        Write-Verbose -Verbose "Starting tgz package creation for architecture '$Architecture'"
        if ($Architecture -eq 'current') {
            throw 'Building a tgz package requires a specific architecture targeting Linux or macOS'
        }
        if ($Architecture -match 'windows') {
            throw "Invalid architecture '$Architecture' - can only build tgz package for Linux or macOS target"
        }
        if (-not $IsLinux -and -not $IsMacOS) {
            throw "Unsupported platform for tgz package - must be packaged on Linux or macOS"
        }

        if ($null -eq $BuildData) {
            $BuildData = Import-DscBuildData
        }
        if ($null -eq $ArtifactDirectory) {
            $artifactDirectory   = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        }

        $filesForPackage = if ($Architecture -match 'linux') {
            $BuildData.PackageFiles.Linux
        } else {
            $BuildData.PackageFiles.MacOS
        }

        $tgzTarget           = $artifactDirectory.TgzTarget
        $bin                 = $artifactDirectory.Bin
        $productVersion      = Get-DscCliVersion
        $productArchitecture = if ($architecture -eq 'aarch64-unknown-linux-musl') {
            'aarch64-linux'
        } elseif ($architecture -eq 'x86_64-unknown-linux-musl') {
            'x86_64-linux'
        } else {
            $architecture
        }
        $packageName = "DSC-$productVersion-$productArchitecture.tar.gz"
        $tarFile     = Join-Path $artifactDirectory.BinRoot $packageName
    }

    process {
        if (Test-Path $tgzTarget) {
            Remove-Item $tgzTarget -Recurse -ErrorAction Stop -Force
        }
        New-Item -ItemType Directory $tgzTarget > $null

        foreach ($file in $filesForPackage) {
            if ((Get-Item "$bin\$file") -is [System.IO.DirectoryInfo]) {
                Copy-Item "$bin\$file" "$tgzTarget\$file" -Recurse -ErrorAction Stop
            } else {
                Copy-Item "$bin\$file" $tgzTarget -ErrorAction Stop
            }
        }

        Write-Verbose "Creating tar.gz file for architecture '$Architecture'"
        $tarArguments = @(
            '-czvf'
            $tarFile
            '-C', $tgzTarget
            '.'
        )
        tar @tarArguments
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create tar.gz file"
        }

        # check it's valid
        $out = file $tarFile
        if ($out -notmatch 'gzip compressed data') {
            throw "Invalid tar.gz file"
        }

        Write-Host -ForegroundColor Green "`ntar.gz file is created at $tarFile"
    }
}

function Build-DscPackage {
    [CmdletBinding()]
    param(
        [DscProjectBuildData]$BuildData,
        [ValidateSet(
            'deb',
            'msix',
            'msix-private',
            'msixbundle',
            'rpm',
            'tgz',
            'zip'
        )]
        $packageType,
        [ValidateSet(
            'current',
            'aarch64-pc-windows-msvc',
            'x86_64-pc-windows-msvc',
            'aarch64-apple-darwin',
            'x86_64-apple-darwin',
            'aarch64-unknown-linux-gnu',
            'aarch64-unknown-linux-musl',
            'x86_64-unknown-linux-gnu',
            'x86_64-unknown-linux-musl'
        )]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$UseX64MakeAppx
    )

    begin {
        if ($null -eq $BuildData) {
            $BuildData = Import-DscBuildData
        }
        if ($null -eq $ArtifactDirectory) {
            $artifactDirectory   = Get-ArtifactDirectoryPath -Architecture $Architecture -Release:$Release
        }
        $buildParams = @{
            BuildData = $BuildData
            ArtifactDirectory = $artifactDirectory
            Architecture = $Architecture
            Release = $Release
        }
    }

    process {
        Write-Verbose "Packaging DSC..."
        switch ($packageType) {
            'deb' {
                Build-DscDebPackage @buildParams
            }
            {$_ -in @('msix', 'msix-private', 'msixbundle')} {
                Build-DscMsixPackage @buildParams -PackageType $packageType -UseX64MakeAppx:$UseX64MakeAppx
            }
            'rpm' {
                Build-DscRpmPackage @buildParams
            }
            'tgz' {
                Build-DscTgzPackage @buildParams
            }
            'zip' {
                Build-DscZipPackage @buildParams
            }
            default {
                throw "Unhandled package type '$packageType'"
            }
        }
    }
}
#endregion Package project functions
