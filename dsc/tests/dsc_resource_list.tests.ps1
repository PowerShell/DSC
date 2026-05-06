# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    try {
        $windowWidth = [Console]::WindowWidth
    } catch {
        $consoleUnavailable = $true
    }
}

Describe 'Tests for listing resources' {
    It 'dsc resource list' {
        $resources = dsc resource list | ConvertFrom-Json -Depth 15
        $LASTEXITCODE | Should -Be 0
        $resources | Should -Not -BeNullOrEmpty
        $resources.Count | Should -BeGreaterThan 0
        $resources.type | Should -Contain 'Microsoft.DSC/Assertion'
        $resources.type | Should -Contain 'Microsoft.DSC/Group'
        $resources.type | Should -Contain 'Microsoft/OSInfo'
        ($resources | Where-Object { $_.type -eq 'Microsoft.DSC/Group' }).Kind | Should -BeExactly 'group'
        ($resources | Where-Object { $_.type -eq 'Microsoft/OSInfo' }).Kind | Should -BeExactly 'resource'
        ($resources | Where-Object { $_.type -eq 'Microsoft.DSC/PowerShell' }).Kind | Should -BeExactly 'adapter'
    }

    It 'dsc resource list --tags "<tags>" and --description "<description> work' -TestCases @(
        if ($IsLinux) {
            @{ tags = 'linux'; description = $null; expectedCount = 2; expectedType = @('DSC.PackageManagement/Apt', 'Microsoft/OSInfo') }
        }
        else {
            @{ tags = 'linux'; description = $null; expectedCount = 1; expectedType = 'Microsoft/OSInfo' }
        }
        @{ tags = $null; description = 'operating system'; expectedCount = 1; expectedType = 'Microsoft/OSInfo' }
        @{ tags = 'linux'; description = 'operating system'; expectedCount = 1; expectedType = 'Microsoft/OSInfo' }
        @{ tags = 'notfound'; description = 'operating system'; expectedCount = 0; expectedType = $null }
        @{ tags = 'linux'; description = 'notfound'; expectedCount = 0; expectedType = $null }
        @{ tags = 'notfound'; description = 'notfound'; expectedCount = 0; expectedType = $null }
    ) {
        param($tags, $description, $expectedCount, $expectedType)

        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            # Need to restrict the search as more resources are being added like from PS7
            $env:DSC_RESOURCE_PATH = Split-Path (Get-Command dsc).Source -Parent

            if ($tags -and $description) {
                $resources = dsc resource list --tags $tags --description $description | ConvertFrom-Json
            }
            elseif ($tags) {
                $resources = dsc resource list --tags $tags | ConvertFrom-Json
            }
            else {
                $resources = dsc resource list --description $description | ConvertFrom-Json
            }

            $LASTEXITCODE | Should -Be 0
            $resources.Count | Should -Be $expectedCount
            if ($expectedCount -gt 0) {
                $resources.type | Should -BeExactly $expectedType
            }
        } finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }

    It 'can accept the use of --output-format as a subcommand' {
        $expectedCount = 1
        $expectedType = 'Microsoft/OSInfo'
        $resources = dsc resource list --description "operating system" --output-format pretty-json | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resources.Count | Should -Be $expectedCount
        if ($expectedCount -gt 0) {
            $resources.type | Should -BeExactly $expectedType
        }
    }

    It 'json progress for resource subcommand' {
        dsc -t json -p json resource list -a '*' 2> $TestDrive/ErrorStream.txt
        $LASTEXITCODE | Should -Be 0
        $lines = Get-Content $TestDrive/ErrorStream.txt
        $ProgressMessagesFound = $False
        foreach ($line in $lines) {
            $jp = $line | ConvertFrom-Json
            if ($jp.activity) { # if line is a progress message
                $jp.id | Should -Not -BeNullOrEmpty
                $jp.totalItems | Should -Not -BeNullOrEmpty
                $jp.completedItems | Should -Not -BeNullOrEmpty
                $ProgressMessagesFound = $True
            }
        }
        $ProgressMessagesFound | Should -BeTrue
    }

    It 'Capabilities are returned' {
        $resource = dsc resource list Microsoft.DSC.Debug/Echo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $resource.capabilities.Count | Should -Be 4
        $resource.capabilities | Should -Contain 'get'
        $resource.capabilities | Should -Contain 'set'
        $resource.capabilities | Should -Contain 'test'
        $resource.capabilities | Should -Contain 'export'
    }

    It 'Invalid adapter returns an error' {
        $out = dsc resource list --adapter 'foo*' 2>&1 | Out-String
        $LASTEXITCODE | Should -Be 0
        $out | Should -BeLike "*ERROR*Adapter not found: foo`*"
    }

    It 'Table is not truncated' -Skip:($consoleUnavailable) {
        $output = dsc resource list --output-format table-no-truncate
        $LASTEXITCODE | Should -Be 0
        $foundWideLine = $false
        foreach ($line in $output) {
            if ($line.Length -gt $windowWidth) {
                $foundWideLine = $true
                break
            }
        }
        $foundWideLine | Should -BeTrue
    }

    It 'No duplicates based on type name and version are returned' {
        $resource_manifest = @'
{
    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
    "type": "Microsoft.DSC.Debug/EchoDupe",
    "version": "1.2.3",
    "description": "A duplicate resource for testing",
    "get": {
        "executable": "dscecho",
        "args": [
            {
                "jsonInputArg": "--input",
                "mandatory": true
            }
        ]
    },
    "schema": {
        "command": {
            "executable": "dscecho"
        }
    }
}
'@
        $manifestPath = Join-Path $TestDrive "echoDupeManifest.dsc.resource.json"
        $manifestDupePath = Join-Path $TestDrive "echoDupeManifestDuplicate.dsc.resource.json"
        Set-Content -Path $manifestPath -Value $resource_manifest
        Set-Content -Path $manifestDupePath -Value $resource_manifest

        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $TestDrive + [System.IO.Path]::PathSeparator + $env:PATH
            $resources = dsc resource list | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $resourceGroups = $resources | Group-Object -Property type, version
            foreach ($group in $resourceGroups) {
                $group.Count | Should -Be 1 -Because ($resources | ConvertTo-Json -Depth 20)
            }
        } finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }

    It 'What-if capability is added for resources supporting it for: <resource>' -TestCases @(
        @{ resource = 'Test/WhatIf'; capability = 'SetWhatIf' }
        @{ resource = 'Test/WhatIfArgKind'; capability = 'SetWhatIf' }
        @{ resource = 'Test/WhatIfDelete'; capability = 'DeleteWhatIf' }
        @{ resource = 'Test/WhatIfReturnDiff'; capability = 'SetWhatIf' }
    ) {
        param($resource, $capability)

        $out = dsc resource list $resource | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.Count | Should -Be 1
        $out.capabilities | Should -Contain $capability
    }
}
