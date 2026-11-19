# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$env:DSC_RESOURCE_PATH = $originalResourcePath
$env:DSC_RESTRICTED_PATH = $originalRestrictedPath
$testExecutable = Join-Path $resourceDirectory.FullName 'csp.exe'
Get-Process |
    Where-Object { $_.Path -eq $testExecutable } |
    Wait-Process -Timeout 30 -ErrorAction Stop
