# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Extension Manifests' {
    It 'Extension manifests with condition: <condition>' -TestCases @(
        @{ condition = "[equals(1, 1)]"; shouldBeFound = $true }
        @{ condition = "[equals(1, 0)]"; shouldBeFound = $false }
    ) {
        param($condition, $shouldBeFound)

        $extension_manifest = @"
{
    "`$schema": "https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json",
    "type": "Test/Extension",
    "condition": "$condition",
    "version": "0.1.0",
    "import": {
        "fileExtensions": ["foo"],
        "executable": "dsc"
    }
}
"@

        try {
            $env:DSC_RESOURCE_PATH = $TestDrive
            $manifestPath = Join-Path -Path $TestDrive -ChildPath 'Extension.dsc.extension.json'
            $extension_manifest | Out-File -FilePath $manifestPath -Encoding utf8
            $extensions = dsc extension list | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            if ($shouldBeFound) {
                $extensions.count | Should -Be 1
                $extensions.type | Should -BeExactly 'Test/Extension'
            }
            else {
                $extensions.count | Should -Be 0
            }
        } finally {
            $env:DSC_RESOURCE_PATH = $null
        }
    }
}
