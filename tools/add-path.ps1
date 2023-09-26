# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# This script will add the script directory to the PATH environment variable
# for the current user. This is useful for development purposes.

$pathSeparator = [System.IO.Path]::PathSeparator
$paths = $env:PATH.Split($pathSeparator)
if ($paths -notcontains $PSScriptRoot) {
    $env:PATH = "$PSScriptRoot" + $pathSeparator + $env:PATH
    Write-Host -ForegroundColor Green "Added $PSScriptRoot to `$env:PATH"
}
