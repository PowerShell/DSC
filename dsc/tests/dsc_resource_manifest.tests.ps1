# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Resource Manifests' {
    It 'Resource manifests with condition: <condition>' -TestCases @(
        @{ condition = "[equals(1, 1)]"; shouldBeFound = $true }
        @{ condition = "[equals(1, 0)]"; shouldBeFound = $false }
    ) {
        param($condition, $shouldBeFound)

        $resource_manifest = @"
{
    "`$schema": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
    "type": "Test/MyEcho",
    "version": "1.0.0",
    "condition": "$condition",
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
"@

        try {
            $env:DSC_RESOURCE_PATH = $TestDrive
            $manifestPath = Join-Path -Path $TestDrive -ChildPath 'MyEcho.dsc.resource.json'
            $resource_manifest | Out-File -FilePath $manifestPath -Encoding utf8
            $resources = dsc resource list | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            if ($shouldBeFound) {
                $resources.count | Should -Be 1
                $resources.type | Should -BeExactly 'Test/MyEcho'
            }
            else {
                $resources.count | Should -Be 0
            }
        } finally {
            $env:DSC_RESOURCE_PATH = $null
        }
    }
}
