# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [Parameter()]
    [switch]$RelativePath
)

Get-ChildItem -Path $PSScriptRoot/resources/*.json | ForEach-Object {
    $resource = [pscustomobject]@{
        resourceManifestPath = if ($RelativePath) {
            Resolve-Path -Path $_.FullName -Relative
        } else {
            $_.FullName
        }
    }
    $resource | ConvertTo-Json -Compress
}
