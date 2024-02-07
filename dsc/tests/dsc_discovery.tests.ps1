# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for resource discovery' {
    BeforeAll {
        $env:DSC_RESOURCE_PATH = $testdrive
    }

    AfterEach {
        Remove-Item -Path "$testdrive/test.dsc.resource.*" -ErrorAction SilentlyContinue
    }

    AfterAll {
        $env:DSC_RESOURCE_PATH = $null
    }

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

        Set-Content -Path "$testdrive/test.dsc.resource.json" -Value $resourceJson
        $resources = dsc resource list | ConvertFrom-Json
        $resources.Count | Should -Be 1
        $resources.type | Should -BeExactly 'DSC/TestPathResource'
    }

    It 'support discovering <extension>' -TestCases @(
        @{ extension = 'yaml' }
        @{ extension = 'yml' }
    ) {
        param($extension)

        $resourceYaml = @'
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json
        type: DSC/TestYamlResource
        version: 0.1.0
        get:
          executable: dsc
'@

        Set-Content -Path "$testdrive/test.dsc.resource.$extension" -Value $resourceYaml
        $resources = dsc resource list | ConvertFrom-Json
        $resources.Count | Should -Be 1
        $resources.type | Should -BeExactly 'DSC/TestYamlResource'
    }
}
