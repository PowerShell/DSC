# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'OCI RegistryList export' {
    BeforeEach {
        $previousHome = $env:HOME
        $previousLocal = $env:LOCALAPPDATA
        $previousResourcePath = $env:DSC_RESOURCE_PATH
        $env:HOME = $TestDrive
        $env:LOCALAPPDATA = $TestDrive
        $env:DSC_RESOURCE_PATH = Join-Path $TestDrive manifests
        New-Item -ItemType Directory -Path $env:DSC_RESOURCE_PATH -Force | Out-Null
        Copy-Item "$PSScriptRoot\..\registrylist.dsc.resource.json" $env:DSC_RESOURCE_PATH
    }
    AfterEach {
        $env:HOME = $previousHome
        $env:LOCALAPPDATA = $previousLocal
        $env:DSC_RESOURCE_PATH = $previousResourcePath
    }
    It 'exports an empty selected list as one resource instance' {
        $output = dsc resource export -r Microsoft.Registry.OpenContainerInitiative/RegistryList 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $configuration = $output | ConvertFrom-Json
        $configuration.resources | Should -HaveCount 1
        $configuration.resources[0].name | Should -BeExactly 'OCI Registries'
        $configuration.resources[0].properties.PSObject.Properties.Name |
            Should -Not -Contain '_name'
        $configuration.resources[0].properties.scope | Should -BeExactly currentUser
        $configuration.resources[0].properties.registries | Should -HaveCount 0
    }
    It 'returns the export-only name at the top level of the resource output' {
        $export = oci_registry export 2>$TestDrive\error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $export._name | Should -BeExactly 'OCI Registries'
        $export.registries | Should -HaveCount 0
        $get = oci_registry get 2>$TestDrive\error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $get.PSObject.Properties.Name | Should -Not -Contain '_name'
    }
}
