# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [Parameter()]
    [string]$extensions
)

$fileextensions = [System.Collections.Generic.List[string]]::new()
foreach ($extension in $extensions.Split(',')) {
    $fileextensions.Add('*' + $extension)
}

$packages = Get-AppxPackage
foreach ($package in $packages) {
    $manifests = Get-ChildItem -Path "$($package.InstallLocation)\*" -File -Include $fileextensions -ErrorAction Ignore
    foreach ($manifest in $manifests) {
        @{ manifestPath = $manifest.FullName } | ConvertTo-Json -Compress
    }
}
