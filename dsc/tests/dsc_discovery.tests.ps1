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

    It 'does not support discovering a file with an extension that is not json or yaml' {
        param($extension)

        $resourceInput = @'
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json
        type: DSC/TestYamlResource
        version: 0.1.0
        get:
          executable: dsc
'@

        Set-Content -Path "$testdrive/test.dsc.resource.txt" -Value $resourceInput
        $resources = dsc resource list | ConvertFrom-Json
        $resources.Count | Should -Be 0
    }

    It 'warns on invalid semver' {
        $manifest = @'
        {
            "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json",
            "type": "Test/InvalidSemver",
            "version": "1.1.0..1",
            "get": {
                "executable": "dsctest"
            },
            "schema": {
                "command": {
                    "executable": "dsctest"
                }
            }
        }
'@
        $oldPath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = $testdrive
            Set-Content -Path "$testdrive/test.dsc.resource.json" -Value $manifest
            $out = dsc resource list 2>&1
            write-verbose -verbose ($out | Out-String)
            $out | Should -Match 'WARN.*?Validation.*?Invalid manifest.*?version'
        }
        finally {
            $env:DSC_RESOURCE_PATH = $oldPath
        }
    }
}
