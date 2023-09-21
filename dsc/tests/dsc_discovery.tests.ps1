# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for resource discovery' {
    It 'Use DSC_RESOURCE_PATH instead of PATH when defined' {
        $resourceJson = @'
        {
            "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json",
            "type": "DSC/TestPathResource",
            "version": "0.1.0",
            "get": {
              "executable": "dsc"
            }
          }
'@

        try {
            $env:DSC_RESOURCE_PATH = $testdrive
            Set-Content -Path "$testdrive/test.dsc.resource.json" -Value $resourceJson
            $resources = dsc resource list | ConvertFrom-Json
            $resources.Count | Should -Be 1
            $resources.type | Should -BeExactly 'DSC/TestPathResource'
        }
        finally {
            $env:DSC_RESOURCE_PATH = $null
        }
    }
}
