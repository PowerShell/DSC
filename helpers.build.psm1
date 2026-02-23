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
    [bool]   $ClippyUnclean
    [bool]   $ClippyPedanticUnclean
    [bool]   $SkipTestProject
    [bool]   $OperatingSystemCheck
    [bool]   $TestOnly
    [DscSupportedPlatformOS] $SupportedPlatformOS = 7
    [string[]] $Binaries
    [DscProjectCopyFiles] $CopyFiles
    [DscProjectSkipTest] $SkipTest

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
            $channel = 'ms-stable'
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

    process {
        Write-Verbose -Verbose "Installing clippy..."
        if ($UseCFS) {
            cargo install clippy --config .cargo/config.toml
        } else {
            if ($Architecture -ne 'current') {
                write-verbose -verbose "Installing clippy for $Architecture"
                rustup component add clippy --target $Architecture
            } else {
                write-verbose -verbose "Installing clippy for current architecture"
                rustup component add clippy
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

function Install-Protobuf {
    <#
        .SYNOPSIS
        Installs Protobuf for the protoc executable.
    #>

    [cmdletbinding()]
    param()

    process {
        if (Test-CommandAvailable -Name 'protoc') {
            Write-Verbose "Protobuf already installed."
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

                Write-Host "Fetching latest Protocol Buffers release info..."
                $release = Invoke-RestMethod -Uri "https://api.github.com/repos/protocolbuffers/protobuf/releases/latest"
                $asset = $release.assets | Where-Object { $_.name -match "protoc-.*-$arch\.zip" }
                if (-not $asset) { throw "No matching protoc binary found for $arch" }
                $downloadUrl = $asset.browser_download_url
                $zipPath = "$env:TEMP\protoc.zip"

                Write-Host "Downloading protoc from $downloadUrl..."
                Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
                $installDir = "$env:USERPROFILE\protoc"
                if (-not (Test-Path $installDir)) { New-Item -ItemType Directory -Path $installDir | Out-Null }

                Write-Host "Extracting protoc to $installDir..."
                Expand-Archive -Path $zipPath -DestinationPath $installDir -Force

                $envPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
                if ($envPath -notlike "*$installDir\bin*") {
                    $env:PATH += ";$installDir\bin"
                }

                Write-Host "Verifying protoc installation..."
                & "$installDir\bin\protoc.exe" --version

                Write-Host "✅ Protocol Buffers installed successfully!"
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
            if (Test-CommandAvailable -Name 'apt') {
                Write-Verbose -Verbose "Using apt to install Protobuf"
                sudo apt install -y protobuf-compiler
                Write-Verbose -Verbose (Get-Command protoc | Out-String)
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
                Register-PSResourceRepository -uri 'https://pkgs.dev.azure.com/powershell/PowerShell/_packaging/powershell/nuget/v2' -Name CFS -Trusted
            }
        }
    }

    process {
        if ($IsWindows) {
            # PSDesiredStateConfiguration module is needed for Microsoft.Windows/WindowsPowerShell adapter
            $FullyQualifiedName = @{ModuleName="PSDesiredStateConfiguration";ModuleVersion="2.0.7"}
            if (-not(Get-Module -ListAvailable -FullyQualifiedName $FullyQualifiedName))
            {
                Install-PSResource -Name PSDesiredStateConfiguration -Version 2.0.7 -Repository $repository -TrustRepository
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
function Set-RustChannel {
    <#
        .SYNOPSIS
        Sets the rust default toolchain to the stable channel.
    #>

    [CmdletBinding()]
    param()

    begin {
        $rustup, $channel = Get-RustUp
    }

    process {
        & $rustup default stable
    }
}

function Set-CargoEnvironment {
    <#
        .SYNOPSIS
        Defines environment variables for Cargo to enable retrieving crates.
    #>
    [cmdletbinding()]
    param(
        [switch]$UseCFS,
        [switch]$UseCFSAuth
    )

    process {
        if ($UseCFS) {
            Write-Host "Using CFS for cargo source replacement"
            ${env:CARGO_SOURCE_crates-io_REPLACE_WITH} = $null
            $env:CARGO_REGISTRIES_CRATESIO_INDEX = $null

            if ($UseCFSAuth) {
                if ($null -eq (Get-Command 'az' -ErrorAction Ignore)) {
                    throw "Azure CLI not found"
                }

                if ($null -ne (Get-Command az -ErrorAction Ignore)) {
                    Write-Host "Getting token"
                    $accessToken = az account get-access-token --query accessToken --resource 499b84ac-1321-427f-aa17-267ca6975798 -o tsv
                    if ($LASTEXITCODE -ne 0) {
                        Write-Warning "Failed to get access token, use 'az login' first, or use '-useCratesIO' to use crates.io.  Proceeding with anonymous access."
                    } else {
                        $header = "Bearer $accessToken"
                        $env:CARGO_REGISTRIES_POWERSHELL_TOKEN = $header
                        $env:CARGO_REGISTRIES_POWERSHELL_CREDENTIAL_PROVIDER = 'cargo:token'
                        $env:CARGO_REGISTRIES_POWERSHELL_INDEX = "sparse+https://pkgs.dev.azure.com/powershell/PowerShell/_packaging/powershell~force-auth/Cargo/index/"
                    }
                }
                else {
                    Write-Warning "Azure CLI not found, proceeding with anonymous access."
                }
            }
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

        if ($Clippy -and !$Project.ClippyUnclean) {
            $clippyFlags = @()
            if (!$Project.ClippyPedanticUnclean) {
                cargo clippy @clippyFlags --% -- -Dclippy::pedantic --no-deps -Dwarnings
            } else {
                cargo clippy @clippyFlags --% -- -Dwarnings --no-deps
            }

            if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
                throw "Last exit code is $LASTEXITCODE, clippy failed for at least one project"
            }
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

#region    Test project functions
function Test-RustProject {
    [CmdletBinding()]
    param(
        [DscProjectDefinition[]]$Project,
        [ValidateSet('current','aarch64-pc-windows-msvc','x86_64-pc-windows-msvc','aarch64-apple-darwin','x86_64-apple-darwin','aarch64-unknown-linux-gnu','aarch64-unknown-linux-musl','x86_64-unknown-linux-gnu','x86_64-unknown-linux-musl')]
        $Architecture = 'current',
        [switch]$Release,
        [switch]$Docs
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
        cargo test @flags

        if ($null -ne $LASTEXITCODE -and $LASTEXITCODE -ne 0) {
            Write-Error "Last exit code is $LASTEXITCODE, rust tests failed"
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
        [switch]$Release
    )

    begin {
        if ($IsWindows) {
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
        $makeappx = Find-MakeAppx
        $makepri  = Get-Item (Join-Path $makeappx.Directory "makepri.exe") -ErrorAction Stop
    }

    process {
        if ($packageType -eq 'msixbundle') {
            $packageName = "DSC-$productVersion-Win"
            $msixArguments = @(
                'bundle'
                '/d', $artifactDirectory.MsixBundle
                '/p', "$($artifactDirectory.Bin)\$packageName.msixbundle"
            )
            & $makeappx @msixArguments
            return
        }

        if ($architecture -eq 'current') {
            throw 'MSIX requires a specific architecture'
        }

        $displayName = 'DesiredStateConfiguration'
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
                $displayName += "-Private"
            }
            else {
                $displayName += "-Preview"
            }
        } else {
            # appx requires a version in the format of major.minor.build.revision with revision being 0
            $productVersion += ".0"
        }

        Write-Verbose -Verbose "Product version is $productVersion"
        $arch = ($architecture -eq 'aarch64-pc-windows-msvc') ? 'arm64' : 'x64'

        # Appx manifest needs to be in root of source path, but the embedded version needs to be updated
        # cp-459155 is 'CN=Microsoft Windows Store Publisher (Store EKU), O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
        # authenticodeFormer is 'CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
        $releasePublisher = 'CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US'
        # Retrieve manifest and set version correctly
        $appxManifest = Get-Content "$PSScriptRoot\packaging\msix\AppxManifest.xml" -Raw
        $appxManifest = $appxManifest.Replace('$VERSION$', $ProductVersion).Replace('$ARCH$', $Arch).Replace('$PRODUCTNAME$', $productName).Replace('$DISPLAYNAME$', $displayName).Replace('$PUBLISHER$', $releasePublisher)
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
        Write-Host -ForegroundColor Green "`nMSIX package is created at $packageName"
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
            'msix',
            'msix-private',
            'msixbundle',
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
        [switch]$Release
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
        if ($packageType -match 'msix') {
            Build-DscMsixPackage @buildParams -PackageType $packageType
        } elseif ($packageType -eq 'tgz') {
            Build-DscTgzPackage @buildParams
        } elseif ($packageType -eq 'zip') {
            Build-DscZipPackage @buildParams
        } else {
            throw "Unhandled package type '$packageType'"
        }
    }
}
#endregion Package project functions
