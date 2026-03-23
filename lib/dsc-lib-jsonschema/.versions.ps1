#!/usr/bin/env pwsh

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

<#
    .SYNOPSIS
    Generate the version info for the DSC project.

    .DESCRIPTION
    This script inspects the git tags for the project and uses them to construct data representing
    those versions, saving it to `.versions.json` in the same directory as this script.

    The data file contains every non-prerelease version tag as well as the latest major, minor, and
    patch version releases.

    The versions are saved as:

    - `V<Major>`, like `V3`, for every major version number.
    - `V<Major>_<Minor>`, like `V3_1`, for every minor version number.
    - `V<Major>_<Minor>_<Patch>`, like `V3_1_0`, for every non-prerelease version number.

    This data is used by `build.rs` to generate the contents for the `RecognizedSchemaVersion`
    enum type definition and trait implementations.
#>

[CmdletBinding()]
param()

begin {
    function Get-DscProjectTagVersion {
        [cmdletbinding()]
        [OutputType([semver])]
        param()

        process {
            $allOut = git fetch --all --tags *>&1

            if ($LASTEXITCODE -ne 0) {
                throw "Unable to fetch git tags:`n`t$($allOut -join "`n`t")"
            }
            
            git tag -l
            | Where-Object -FilterScript {$_ -match '^v\d+(\.\d+){2}$' }
            | ForEach-Object -Process { [semver]($_.Substring(1)) }
        }
    }

    function ConvertTo-EnumName {
        [CmdletBinding()]
        [OutputType([string])]
        param(
            [Parameter(ValueFromPipeline)]
            [semver[]]$Version,
            [switch]$Major,
            [switch]$Minor

        )

        process {
            foreach ($v in $Version) {
                if ($Major) {
                    'V{0}' -f $v.Major
                } elseif ($Minor) {
                    'V{0}_{1}' -f $v.Major, $v.Minor
                } else {
                    'V{0}_{1}_{2}' -f $v.Major, $v.Minor, $v.Patch
                }
            }
        }
    }

    function Export-DscProjectTagVersion {
        [cmdletbinding()]
        param()

        process {
            $publishedVersions = Get-DscProjectTagVersion
            | Sort-Object -Descending

            [System.Collections.Generic.HashSet[semver]]$majorVersions = @()
            [System.Collections.Generic.HashSet[semver]]$minorVersions = @()
            [System.Collections.Generic.HashSet[semver]]$patchVersions = @()

            foreach ($version in $publishedVersions) {
                $null = $majorVersions.Add([semver]"$($version.Major)")
                $null = $minorVersions.Add([semver]"$($version.Major).$($version.Minor)")
                $null = $patchVersions.Add($version)
            }

            # Sort the versions with major version, then each child minor version and child patch versions.
            [System.Collections.Generic.HashSet[string]]$allVersions = @()
            foreach ($major in ($majorVersions | Sort-Object -Descending)) {
                $null = $allVersions.Add(($major | ConvertTo-EnumName -Major))

                $majorMinor = $minorVersions
                | Where-Object { $_.Major -eq $major.Major }
                | Sort-Object -Descending

                foreach ($minor in $majorMinor) {
                    $null = $allVersions.Add(($minor | ConvertTo-EnumName -Minor))

                    $majorMinorPatch = $patchVersions
                    | Where-Object { $_.Major -eq $minor.Major -and $_.Minor -eq $minor.Minor }
                    | Sort-Object -Descending

                    foreach ($patch in $majorMinorPatch) {
                        $null = $allVersions.Add(($patch | ConvertTo-EnumName))
                    }
                }
            }

            [string]$latestMajorVersion = $majorVersions
            | Sort-Object -Descending
            | Select-Object -First 1
            | ConvertTo-EnumName -Major
            [string]$latestMinorVersion = $minorVersions
            | Sort-Object -Descending
            | Select-Object -First 1
            | ConvertTo-EnumName -Minor
            [string]$latestPatchVersion = $patchVersions
            | Sort-Object -Descending
            | Select-Object -First 1
            | ConvertTo-EnumName

            $data = [ordered]@{
                latestMajor = $latestMajorVersion
                latestMinor = $latestMinorVersion
                latestPatch = $latestPatchVersion
                all = $allVersions
            }

            $dataJson = $data
            | ConvertTo-Json
            | ForEach-Object -Process { $_ -replace "`r`n", "`n"}

            $dataPath = Join-Path -Path $PSScriptRoot -ChildPath '.versions.json'
            $dataContent = Get-Content -Raw -Path $dataPath

            if ($dataJson.Trim() -ne $dataContent.Trim()) {
                $dataJson | Set-Content -Path $PSScriptRoot/.versions.json
            }

            $dataJson
        }
    }
}

process {
    Export-DscProjectTagVersion
}