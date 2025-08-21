# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }

$manifests = $psPaths | ForEach-Object -Parallel {
    $searchPatterns = @('*.dsc.resource.json', '*.dsc.resource.yaml', '*.dsc.resource.yml')
    $enumOptions = [System.IO.EnumerationOptions]@{ IgnoreInaccessible = $false; RecurseSubdirectories = $true }
    foreach ($pattern in $searchPatterns) {
        try {
            [System.IO.Directory]::EnumerateFiles($_, $pattern, $enumOptions) | ForEach-Object {
                @{ manifestPath = $_ }
            }
        } catch { }
    }
} -ThrottleLimit 10

$manifests | ForEach-Object { $_ | ConvertTo-Json -Compress }