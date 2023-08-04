# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

# This script will add the current directory to the PATH environment variable
# for the current user. This is useful for development purposes.

$pathSeparator = [System.IO.Path]::PathSeparator
$paths = $env:PATH.Split($pathSeparator)
if (-not $paths -contains $PWD) {
    $env:PATH = $PWD + $pathSeparator + $env:PATH
    Write-Host -ForegroundColor Green "Added $PWD to PATH"
}
