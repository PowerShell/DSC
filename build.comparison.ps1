#!/usr/bin/env pwsh

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
<#
    .SYNOPSIS
    Compares build performance for the existing and new build scripts.

    .DESCRIPTION
    This is a temporary comparison tool to help us determine when we're ready to switch from the
    legacy build script to the new one. It primarily currently focuses on performance, but can be
    extended to cover correctness as well.

    Currently, the only correctness check is for comparing output files in the bin folder, expecting
    both builds to produce the same artifacts.

    In local testing (only building, excluding tests and packaging):

    - Average cold builds show an improvement of ~2x, averaging 4-5m for new and 8-10m for legacy.
    - Average warm builds show an improvement of ~20x, averaging 10-15s for new and 180-240s for legacy.
    - Average hot builds show an improvement of ~10x, averaging 10-15s for new and 120-150s for legacy.

    Note that the difference between hot and warm is minimal with the new script because it takes
    advantage of the shared dependencies and concurrent compilation. For Legacy, the difference is
    more noticeable. Even for cold builds, the new script cuts build times in half.

    Except for ComparisonKind and Quiet, all other parameters are the same as for the legacy build
    script.

    .PARAMETER ComparisonKind
    Defines how to compare the builds. Valid options are:

    - `Cold` (default): Before invoking either build script, delete both the `bin` and `target`
      folders to ensure no use of cached build artifacts.
    - `Warm`: Initially build the project to cache build artifacts in `target`. Then clean the
      `bin` directory before invoking each build script.
    - `Hot`: Same as `Warm`, but with `sccache` enabled (only tested on Windows).

    .PARAMETER Quiet
    Defines whether to run the comparison with build output or not. In Quiet mode, only the progress
    bars for the scripts are shown. All output for the build scripts is redirected to null.
#>
[CmdletBinding()]
param(
    [ValidateSet('Cold', 'Warm', 'Hot')]
    [string]$ComparisonKind = 'Cold',
    [switch]$Quiet,
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
    [switch]$GetPackageVersion,
    [switch]$SkipLinkCheck,
    [switch]$UseX64MakeAppx,
    [switch]$UseCFS,
    [switch]$UpdateLockFile,
    [switch]$Audit,
    [switch]$UseCFSAuth,
    [switch]$Clean
)

begin {
    $buildParams = [hashtable]$PSBoundParameters
    if ($buildParams.ContainsKey('ComparisonKind')) {
        $buildParams.Remove('ComparisonKind')
    }
    if ($buildParams.ContainsKey('Quiet')) {
        $buildParams.Remove('Quiet')
    }

    function Show-BuildParameters {
        [cmdletbinding()]
        param(
            [hashtable]$BuildParams
        )
        $sb = [System.Text.StringBuilder]::new()
        $sb.Append("[") > $null
        $padTo = 0
        foreach ($key in $BuildParams.Keys) {
            if ($key.Length -ge $padTo) {
                $padTo = $key.Length + 1
            }
        }
        foreach ($key in $BuildParams.Keys) {
            $sb.Append("`n`t").
                Append($key).
                Append((' ' * ($padTo - $key.Length))).
                Append(': ').
                Append($BuildParams[$key].ToString()) > $null
        }

        $sb.Append("`n]") > $null
        $sb.ToString()
    }

    function Write-WithTimeStamp {
        [cmdletbinding()]
        param(
            [string]$Message,
            [System.ConsoleColor]$Color = 'Cyan'
        )

        $timestampFormat   = "yyyy-MM-ddTHH:mm:ss.fff"
        $Message = '{0} - {1}' -f @(
            (Get-Date -AsUTC).ToString($timestampFormat)
            $Message
        )
        Write-Host -ForegroundColor $Color $Message
    }
    function Write-Divider {
        param(
            [System.ConsoleColor]$Color = 'Cyan'
        )

        Write-Host -ForegroundColor $Color ('-' * 80)
    }

    function Measure-Build {
        [CmdletBinding()]
        param(
            [ValidateSet('Cold', 'Warm', 'Hot')]
            [string]$ComparisonKind = 'Cold',
            [hashtable]$BuildParams,
            [switch]$Quiet
        )
        begin {
            # Define variables
            $durationFormat    = "mm\:ss\.fff"
            $legacyScript      = Join-Path $PSScriptRoot "build.ps1"
            $legacyBuildParams = $BuildParams.Clone()
            $newScript         = Join-Path $PSScriptRoot "build.new.ps1"
            $newBuildParams    = $BuildParams.Clone()
            $binFolder         = Join-Path $PSScriptRoot "bin"
            $targetFolder      = Join-Path $PSScriptRoot "target"
            # Prerun steps
            Write-Divider
            Write-WithTimeStamp "Starting $ComparisonKind comparison"
            Write-Divider

            $priorWrapper = $env:RUSTC_WRAPPER
            $usingWrapper = -not [string]::IsNullOrEmpty($priorWrapper)
            if ($ComparisonKind -eq 'Hot' -and -not $usingWrapper) {
                if ($sccache = Get-Command -Name sccache -CommandType Application -ErrorAction Ignore) {
                    Write-WithTimeStamp "Using sccache for Hot comparison"
                    $env:RUSTC_WRAPPER = $sccache.Source
                    Write-WithTimeStamp "Set RUSTC_WRAPPER to: $env:RUSTC_WRAPPER"
                } else {
                    Write-WithTimeStamp -Color Magenta (@(
                        "Unable to use sccache for Hot comparison, sccache not found."
                        'Install `sccache` and ensure it is available in PATH.'
                        'Continuing as a Warm comparison instead.'
                    ) -join ("`n" + (' ' * 25)))
                    $ComparisonKind = 'Warm'
                }
            } elseif ($ComparisonKind -eq 'Hot' -and $usingWrapper) {
                Write-WithTimeStamp "Using previously configured cache wrapper for Hot comparison"
                Write-WithTimeStamp "Set RUSTC_WRAPPER to: $env:RUSTC_WRAPPER"
            } elseif ($usingWrapper) {
                Write-WithTimeStamp "Temporarily disabling previously configured cache wrapper for $ComparisonKind comparison"
                $env:RUSTC_WRAPPER = $null
                Write-WithTimeStamp "Set RUSTC_WRAPPER to null."
            }

            if ($ComparisonKind -eq 'Cold') {
                Write-WithTimeStamp "Ensuring clean build conditions before comparison..."
                if (Test-Path $binFolder) {
                    Remove-Item $binFolder -Recurse -Force
                    Write-WithTimeStamp "Removed bin folder."
                }
                if (Test-Path $targetFolder) {
                    Remove-Item $targetFolder -Recurse -Force
                    Write-WithTimeStamp "Removed target folder."
                }
            } else {
                Write-WithTimeStamp "Pre-building project before $ComparisonKind comparison..."
                Write-Divider
                if ($Quiet) {
                    & $newScript @newBuildParams *>$null
                } else {
                    & $newScript @newBuildParams
                }
                Write-Divider
                Write-WithTimeStamp "Finished pre-build"
                Write-Divider
            }
        }

        process {
            #region New script
            Write-Divider -Color Yellow
            $message = 'Building with new script with parameters: {0}' -f (Show-BuildParameters $newBuildParams)
            Write-WithTimeStamp -Color Yellow $message
            Write-Divider -Color Yellow
            $newStart = Get-Date -AsUTC
            if ($Quiet) {
                & $newScript @newBuildParams *>$null
            } else {
                & $newScript @newBuildParams
            }
            $newEnd = Get-Date -AsUTC
            $newRun = $newEnd - $newStart
            Write-Divider -Color Yellow
            $message = 'Built with new script ({0})' -f $newRun.ToString($durationFormat)
            Write-WithTimeStamp -Color Yellow $message
            Write-Divider -Color Yellow
            $newBinFiles = Get-ChildItem $binFolder -Recurse -File | ForEach-Object {
                Resolve-Path -Relative -RelativeBasePath $binFolder -Path $_
            }
            #endregion new script
            #region Between executions
            Write-Divider
            if ($ComparisonKind -eq 'Cold') {
                
                Write-WithTimeStamp "Ensuring clean build conditions before for Cold comparison..."
                if (Test-Path $binFolder) {
                    Remove-Item $binFolder -Recurse -Force > $null
                    Write-WithTimeStamp "Removed bin folder."
                }
                if (Test-Path $targetFolder) {
                    Remove-Item $targetFolder -Recurse -Force > $null
                    Write-WithTimeStamp "Removed target folder."
                }
            } else {
                Write-WithTimeStamp "Removing bin folder before $ComparisonKind comparison..."
                if (Test-Path $binFolder) {
                    Remove-Item $binFolder -Recurse -Force > $null
                    Write-WithTimeStamp "Removed bin folder."
                }
            }
            Write-Divider
            #endregion Between executions

            #region Legacy script
            Write-Divider -Color Yellow
            $message = 'Building with legacy script with parameters: {0}' -f (Show-BuildParameters $legacyBuildParams)
            Write-WithTimeStamp -Color Yellow $message
            Write-Divider -Color Yellow
            $legacyStart = Get-Date -AsUTC
            if ($Quiet) {
                & $legacyScript @legacyBuildParams *>$null
            } else {
                & $legacyScript @legacyBuildParams
            }
            $legacyEnd = Get-Date -AsUTC
            $legacyRun = $legacyEnd - $legacyStart
            Write-Divider -Color Yellow
            $message = 'Built with legacy script ({0})' -f $newRun.ToString($durationFormat)
            Write-WithTimeStamp -Color Yellow $message
            Write-Divider -Color Yellow
            $legacyBinFiles = Get-ChildItem $binFolder -Recurse -File | ForEach-Object {
                Resolve-Path -Relative -RelativeBasePath $binFolder -Path $_
            }
            #endregion Legacy script

            #region Comparison
            Write-Divider -Color Green
            Write-WithTimeStamp -Color Green "Reporting on builds"
            Write-Divider -Color Green
            $info = if ($newRun -lt $legacyRun) {
                [pscustomobject]@{
                    FasterBuild          = 'New'
                    SlowerBuild          = 'Legacy'
                    ComparisonKind       = $ComparisonKind
                    DifferenceDuration   = $legacyRun - $newRun
                    DifferenceMultiplier = $legacyRun / $newRun
                    NewDuration          = $newRun
                    LegacyDuration       = $legacyRun
                    SameBinFiles         = $null -eq (Compare-Object $newBinFiles $legacyBinFiles)
                    NewBinFiles          = $newBinFiles
                    LegacyBinFiles       = $legacyBinFiles
                }
            } else {
                [pscustomobject]@{
                    FasterBuild          = 'Legacy'
                    SlowerBuild          = 'New'
                    ComparisonKind       = $ComparisonKind
                    DifferenceDuration   = $newRun - $legacyRun
                    DifferenceMultiplier = $newRun / $legacyRun
                    NewDuration          = $newRun
                    LegacyDuration       = $legacyRun
                    SameBinFiles         = $null -eq (Compare-Object $newBinFiles $legacyBinFiles)
                    NewBinFiles          = $newBinFiles
                    LegacyBinFiles       = $legacyBinFiles
                }
            }
            Write-WithTimeStamp -Color Green ('{0} build script was {1:n2}x faster than {2}' -f @(
                $info.FasterBuild
                $info.DifferenceMultiplier
                $info.SlowerBuild
            ))
            
            Write-WithTimeStamp -Color Green "Details:`n`n$(($info | Format-List * -Force | Out-String).Trim())`n"
            Write-Divider -Color Green
            #endregion Comparison
        }

        clean {
            if ($priorWrapper -or $ComparisonKind -eq 'Hot') {
                Write-Divider
                Write-WithTimeStamp "Cleaning up"
                Write-Divider
                if ($priorWrapper) {
                    Write-WithTimeStamp "Resetting RUSTC_WRAPPER to prior wrapper"
                    $env:RUSTC_WRAPPER = $priorWrapper
                    Write-WithTimeStamp "Reset RUStC_WRAPPER to '$env:RUSTC_WRAPPER'"
                } elseif ($ComparisonKind -eq 'Hot') {
                    Write-WithTimeStamp "Removing RUSTC_WRAPPER"
                    $env:RUSTC_WRAPPER = $null
                    Write-WithTimeStamp "Removed RUSTC_WRAPPER"
                }
                Write-Divider
                Write-WithTimeStamp "Cleanup Finished"
                Write-Divider
            }
        }
    }
}

process {
    Measure-Build -ComparisonKind $ComparisonKind -BuildParams $buildParams -Quiet:$Quiet
}