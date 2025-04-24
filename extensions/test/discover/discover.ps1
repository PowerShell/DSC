# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Get-ChildItem -Path $PSScriptRoot/resources/*.json | ForEach-Object {
    $resource = [pscustomobject]@{
        resourceManifestPath = $_.FullName
    }
    $resource | ConvertTo-Json -Compress
}
