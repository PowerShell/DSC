# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'OCI RegistryList get' {
    BeforeDiscovery {
        function Test-IsolatedMachineSettings {
            $IsWindows
        }
        $canIsolateMachineSettings = Test-IsolatedMachineSettings
    }
    BeforeEach {
        $previousHome = $env:HOME
        $previousLocal = $env:LOCALAPPDATA
        $previousResourcePath = $env:DSC_RESOURCE_PATH
        $previousMachine = $env:ProgramData
        $env:HOME = $TestDrive
        $env:LOCALAPPDATA = $TestDrive
        $env:ProgramData = Join-Path $TestDrive ([guid]::NewGuid().ToString())
        $env:DSC_RESOURCE_PATH = Join-Path $TestDrive manifests
        New-Item -ItemType Directory -Path $env:DSC_RESOURCE_PATH -Force | Out-Null
        Copy-Item "$PSScriptRoot\..\registrylist.dsc.resource.json" $env:DSC_RESOURCE_PATH
    }
    AfterEach {
        $env:HOME = $previousHome
        $env:LOCALAPPDATA = $previousLocal
        $env:DSC_RESOURCE_PATH = $previousResourcePath
        $env:ProgramData = $previousMachine
    }
    It 'returns an empty currentUser preference when no settings file exists' {
        $result = '{}' | dsc resource get -r Microsoft.Registry.OpenContainerInitiative/RegistryList -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $state = ($result | ConvertFrom-Json).actualState
        $state.scope | Should -BeExactly currentUser
        $state.policy | Should -BeFalse
        $state.registries | Should -HaveCount 0
    }
    It 'rejects currentUser policy' {
        '{"policy":true}' | dsc resource get -r Microsoft.Registry.OpenContainerInitiative/RegistryList -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Not -Be 0
    }
    It 'reads the selected allUsers file with policy <Policy>' -ForEach @(
        @{ Policy = $false }
        @{ Policy = $true }
    ) -Skip:(-not $canIsolateMachineSettings) {
        $folder = Join-Path $env:ProgramData dsc
        New-Item -ItemType Directory -Path $folder -Force | Out-Null
        $suffix = if ($Policy) { 'policy' } else { 'settings' }
        '{"registries":[{"machine":{"location":"machine.example"}}]}' |
            Set-Content -LiteralPath (Join-Path $folder "Microsoft.Registry.OpenContainerInitaitive.$suffix.json")
        $inputJson = @{ scope = 'allUsers'; policy = $Policy } | ConvertTo-Json
        $result = $inputJson | dsc resource get -r Microsoft.Registry.OpenContainerInitiative/RegistryList -f - 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive\error.log)
        $state = ($result | ConvertFrom-Json).actualState
        $state.scope | Should -BeExactly allUsers
        $state.policy | Should -Be $Policy
        $state.registries[0].machine.location | Should -BeExactly machine.example
    }
}
