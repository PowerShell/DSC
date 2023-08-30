# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for resource discovery' {
    It 'Use DSC_RESOURCE_PATH instead of PATH when defined' {
        $resourceJson = @'
        {
            "manifestVersion": "1.0",
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
