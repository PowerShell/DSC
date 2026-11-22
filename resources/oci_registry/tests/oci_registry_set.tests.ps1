# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'OCI RegistryList set' {
    BeforeEach {
        $previousHome = $env:HOME
        $previousLocal = $env:LOCALAPPDATA
        $previousResourcePath = $env:DSC_RESOURCE_PATH
        $root = Join-Path $TestDrive ([guid]::NewGuid().ToString())
        $env:HOME = $root
        $env:LOCALAPPDATA = $root
        $env:DSC_RESOURCE_PATH = Join-Path $root manifests
        New-Item -ItemType Directory -Path $env:DSC_RESOURCE_PATH -Force | Out-Null
        Copy-Item "$PSScriptRoot\..\registrylist.dsc.resource.json" $env:DSC_RESOURCE_PATH
        $resource = 'Microsoft.Registry.OpenContainerInitiative/RegistryList'
    }
    AfterEach {
        $env:HOME = $previousHome
        $env:LOCALAPPDATA = $previousLocal
        $env:DSC_RESOURCE_PATH = $previousResourcePath
    }
    It 'persists registry locations and secret references without storing tokens' {
        $json = '{"registries":[{"public":{"location":"example.com"}},{"private":{"location":"private.example","authentication":{"secretName":"access","secretVault":"vault"}}}]}'
        $json | dsc resource set -r $resource -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $output = '{}' | dsc resource get -r $resource -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $state = ($output | ConvertFrom-Json).actualState
        $state.registries | Should -HaveCount 2
        $state.registries[1].private.authentication.secretName | Should -BeExactly access
        $folder = if ($IsWindows) { 'dsc' } else { '.dsc' }
        $settingsPath = Join-Path $root "$folder\Microsoft.Registry.OpenContainerInitaitive.settings.json"
        $file = Get-Content -Raw $settingsPath | ConvertFrom-Json
        @($file.PSObject.Properties.Name) | Should -HaveCount 1
        $file.registries | Should -HaveCount 2
    }
    It 'replaces the list, including clearing every preference' {
        '{"registries":[{"public":{"location":"example.com"}}]}' | dsc resource set -r $resource -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        '{"registries":[]}' | dsc resource set -r $resource -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $output = '{}' | dsc resource get -r $resource -f - 2>$TestDrive\error.log
        ($output | ConvertFrom-Json).actualState.registries | Should -HaveCount 0
    }
    It 'rejects an invalid registry without writing settings' -ForEach @(
        '{"registries":[{"public":{}}]}'
        '{"registries":[{"private":{"location":"example.com","authentication":{}}}]}'
        '{"registries":[{"one":{"location":"example.com"},"two":{"location":"example.com"}}]}'
    ) {
        $_ | dsc resource set -r $resource -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Not -Be 0
        $folder = if ($IsWindows) { 'dsc' } else { '.dsc' }
        Join-Path $root "$folder\Microsoft.Registry.OpenContainerInitaitive.settings.json" |
            Should -Not -Exist
    }

    It 'rejects reserved registry name <FriendlyName> with a specific error' -ForEach @(
        @{ FriendlyName = 'local' }
        @{ FriendlyName = 'LOCAL' }
        @{ FriendlyName = 'LoCaL' }
    ) {
        $json = @{ registries = @(@{ $FriendlyName = @{ location = 'example.com' } }) } |
            ConvertTo-Json -Depth 5
        $json | dsc resource set -r $resource -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -LiteralPath $TestDrive\error.log -Raw |
            Should -Match "Registry friendly name '$FriendlyName' is reserved for locally discovered resources"
        $folder = if ($IsWindows) { 'dsc' } else { '.dsc' }
        Join-Path $root "$folder\Microsoft.Registry.OpenContainerInitaitive.settings.json" |
            Should -Not -Exist
    }
}
