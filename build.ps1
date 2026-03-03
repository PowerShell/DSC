#!/usr/bin/env pwsh

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
using module ./helpers.build.psm1

<#
    .SYNOPSIS
    DSC project build script

    .DESCRIPTION

    .PARAMETER Release
    Determines whether to compile the Rust projects with the release profile. The release profile
    uses significant optimizations for runtime size and speed but compiles much more slowly.

    .PARAMETER Architecture
    Determines which platform architecture to compile for. The default architecture is `current`,
    meaning the current operating system. You can specify an alternate architecture to compile for,
    as Rust supports cross-compilation.

    Valid values are:

    - `current` (default) - The platform matching the current operating system.
    - `aarch64-pc-windows-msvc` - Windows on ARM with MSVC build chain.
    - `x86_64-pc-windows-msvc` - Windows on x86 with MSVC build chain.
    - `aarch64-apple-darwin` - macOS on ARM.
    - `x86_64-apple-darwin` - macOS on x86.
    - `aarch64-unknown-linux-gnu` - Linux on ARM with the glibc build chain.
    - `aarch64-unknown-linux-musl` - Linux on ARM with the musl build chain.
    - `x86_64-unknown-linux-gnu` - Linux on x86 with the glibc build chain.
    - `x86_64-unknown-linux-musl` - Linux on x86 with the musl build chain.

    When packaging, you _must_ specify a specific architecture.

    .PARAMETER Clippy
    Determines whether to lint the Rust projects with Clippy. When you specify this parameter, the
    build script lints the Rust projects before building them. Unlike the legacy script, it still
    produces build artifacts unless a crate fails the linting.

    .PARAMETER SkipBuild
    Determines whether to skip building the project.

    .PARAMETER Test
    Determines whether to run Rust and Pester tests for the project.

    .PARAMETER GetPackageVersion
    Short circuits the build to return the current version of the DSC CLI crate.

    .PARAMETER PackageType
    Determines which package type to create. Must specify a single package type at a time. Valid
    package types are:

    - `msix` - MSIX package, requires a specific architecture.
    - `msix-private` - MSIX private package, requires a specific architecture.
    - `msixbundle` - MSIX bundle package, builds for both Windows targets.
    - `tgz` - Packages the project as a `.tar.gz` file, only for Linux/macOS.
    - `zip` - Packages the project as a `.zip` file, only for Windows.
#>
[CmdletBinding()]
param(
    [switch]$Release,
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
    [switch]$Clippy,
    [switch]$SkipBuild,
    [ValidateSet(
        'msix',
        'msix-private',
        'msixbundle',
        'tgz',
        'zip'
    )]
    $PackageType,
    [switch]$Test,
    [string[]]$Project,
    [switch]$ExcludeRustTests,
    [switch]$ExcludePesterTests,
    [ValidateSet("dsc", "adapters", "extensions", "grammars", "resources")]
    [string[]]$PesterTestGroup,
    [switch]$GetPackageVersion,
    [switch]$SkipLinkCheck,
    [switch]$UseX64MakeAppx,
    [switch]$UseCFS,
    [switch]$UpdateLockFile,
    [switch]$Audit,
    [switch]$UseCFSAuth,
    [switch]$Clean,
    [switch]$CacheRustBuild,
    [switch]$RustDocs,
    [switch]$Quiet
)

begin {
    if ($Quiet) {
        $VerbosePreference = 'SilentlyContinue'
        $InformationPreference = 'SilentlyContinue'
        $ProgressPreference = 'Continue'
    } else {
        $InformationPreference = 'Continue'
        $ProgressPreference = 'SilentlyContinue'
    }

    Import-Module ./helpers.build.psm1 -Force -Verbose:$false
    $usingADO = ($null -ne $env:TF_BUILD)
    if ($usingADO -or $UseCFSAuth) {
        $UseCFS = $true
    }
    # Import the build data
    $BuildData = Import-DscBuildData -RefreshProjects
    # Filter projects if needed.
    if ($Project.Count -ge 1) {
        $BuildData.Projects = $BuildData.Projects | Where-Object -FilterScript {
            $_.Name -in $Project
        }
    }
    $VerboseParam = @{}
    if ($VerbosePreference -eq 'Continue' -and -not $Quiet) {
        $VerboseParam.Verbose = $true
    }

    function Write-BuildProgress {
        [cmdletbinding()]
        param(
            [string]$Activity,
            [string]$Status,
            [switch]$Completed,
            [switch]$Quiet
        )

        process {
            if ($Quiet) {
                $params = [hashtable]$PSBoundParameters
                $params.Remove('Quiet') > $null
                Write-Progress @params
            } elseif ($Completed) {
                Write-Information "Finished build script"
            } else {
                $message = "BUILD:   $Activity"
                if (-not [string]::IsNullOrEmpty($Status)) {
                    $message += "::$Status"
                }
                Write-Information $message
            }
        }
    }
}

process {
    trap {
        Write-Error "An error occurred: $($_ | Out-String)"
        exit 1
    }

    if ($GetPackageVersion) {
        return Get-DscCliVersion @VerboseParam
    }
    $progressParams = @{
        Activity = "Executing build script"
        Quiet    = $Quiet
    }
    Write-BuildProgress @progressParams

    #region    Setup
    $progressParams.Activity = 'Performing setup steps'
    Write-BuildProgress @progressParams
    Write-BuildProgress @progressParams -Status "Determining rustup info"
    $rustup, $channel = Get-RustUp @VerboseParam

    if ($null -ne $PackageType) {
        $SkipBuild = $true
    } else {
        Write-BuildProgress @progressParams -Status 'Configuring Rust environment'
        [hashtable]$priorRustEnvironment = Set-RustEnvironment -CacheRustBuild:$CacheRustBuild @VerboseParam
        Write-BuildProgress @progressParams -Status 'Configuring Cargo environment'
        Set-CargoEnvironment -UseCFS:$UseCFS -UseCFSAuth:$UseCFSAuth @VerboseParam

        # Install or update rust
        if (!$usingADO) {
            Write-BuildProgress @progressParams -Status 'Ensuring Rust is up-to-date'
            Update-Rust @VerboseParam
        }

        if (!$usingADO) {
            Write-BuildProgress @progressParams -Status 'Setting RustUp to default channel'
            $rustup, $channel = Get-RustUp @VerboseParam
            & $rustup default stable
        }

        if ($Clippy) {
            Write-BuildProgress @progressParams -Status 'Ensuring Clippy is available and updated'
            Install-Clippy -UseCFS:$UseCFS -Architecture $Architecture @VerboseParam
        }

        if (-not ($SkipBuild -and $Test -and $ExcludeRustTests)) {
            Write-BuildProgress @progressParams -Status 'Ensuring Protobuf is available'
            Install-Protobuf @VerboseParam

            Write-BuildProgress @progressParams -Status 'Ensuring Node.JS is available'
            Install-NodeJS @VerboseParam

            Write-BuildProgress @progressParams -Status 'Ensuring tree-sitter is available'
            Install-TreeSitter -UseCFS:$UseCFS @VerboseParam
        }
    }

    if (!$SkipBuild -and !$SkipLinkCheck -and $IsWindows) {
        Write-BuildProgress @progressParams -Status "Ensuring Windows C++ build tools are available"
        Install-WindowsCPlusPlusBuildTools @VerboseParam
    }
    #endregion Setup

    if (!$SkipBuild) {
        $progressParams.Activity = 'Building the projects'
        Write-BuildProgress @progressParams
        Write-BuildProgress @progressParams -Status 'Generating grammar bindings'
        Export-GrammarBinding -Project $BuildData.Projects @VerboseParam

        if ($RustDocs) {
            $progressParams.Activity = 'Generating Rust documentation'
            Write-BuildProgress @progressParams

            $docsParams = @{
                Project      = $BuildData.Projects
                Architecture = $Architecture
                Release      = $Release
            }
            Export-RustDocs @docsParams @VerboseParam
        } else {
            $buildParams = @{
                Project      = $BuildData.Projects
                Architecture = $Architecture
                Release      = $Release
                Clean        = $Clean
            }
            Write-BuildProgress @progressParams -Status 'Compiling Rust'
            Build-RustProject @buildParams -Audit:$Audit -Clippy:$Clippy @VerboseParam
            Write-BuildProgress @progressParams -Status "Copying build artifacts"
            Copy-BuildArtifact @buildParams -ExecutableFile $BuildData.PackageFiles.Executable @VerboseParam
        }
    }

    # Ensure PATH includes the output artifacts after building and before testing.
    if ((!$Clippy -and !$SkipBuild) -or $Test) {
        $progressParams.Activity = 'Updating environment variables'
        Write-BuildProgress @progressParams
        Update-PathEnvironment -Architecture $Architecture -Release:$Release @VerboseParam
    }

    if ($Test) {
        $progressParams.Activity = 'Testing projects'
        Write-BuildProgress @progressParams

        if (-not $ExcludeRustTests) {
            $rustTestParams = @{
                Project      = $BuildData.Projects
                Architecture = $Architecture
                Release      = $Release
            }
            Write-BuildProgress @progressParams -Status "Testing Rust projects"
            Test-RustProject @rustTestParams @VerboseParam
        }
        if ($RustDocs) {
            $docTestParams = @{
                Project      = $BuildData.Projects
                Architecture = $Architecture
                Release      = $Release
                Docs         = $true
            }
            Write-BuildProgress @progressParams -Status "Testing documentation for Rust projects"
            Test-RustProject @docTestParams @VerboseParam
        }
        if (-not $ExcludePesterTests) {
            $installParams = @{
                UsingADO = $usingADO
            }
            $pesterParams = @{
                UsingADO = $usingADO
            }
            if ($null -ne $PesterTestGroup) {
                $pesterParams.Group = $PesterTestGroup
            }
            Write-BuildProgress @progressParams -Status "Installing PowerShell test prerequisites"
            Install-PowerShellTestPrerequisite @installParams @VerboseParam
            Write-BuildProgress @progressParams -Status "Invoking pester"
            Test-ProjectWithPester @pesterParams @VerboseParam
        }
    }

    if (-not [string]::IsNullOrEmpty($PackageType)) {
        $progressParams.Activity = "Packaging"
        $packageParams = @{
            BuildData    = $BuildData
            PackageType  = $PackageType
            Architecture = $Architecture
            Release      = $Release
        }
        Write-BuildProgress @progressParams
        Build-DscPackage @packageParams @VerboseParam
    }
}

clean {
    $progressParams.Activity = 'Cleaning up'
    Write-BuildProgress @progressParams

    if ($null -ne $priorRustEnvironment) {
        Write-BuildProgress @progressParams -Status "Restoring rust environment"
        Reset-RustEnvironment -PriorEnvironment $priorRustEnvironment @VerboseParam
    }

    Write-BuildProgress -Completed
}
