# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Registry JSON-RPC operations' -Skip:(!$IsWindows) {
    BeforeAll {
        function Invoke-RegistryRpc {
            param([hashtable[]]$Operation)

            $requests = @(
                @{
                    jsonrpc = '2.0'
                    id = 0
                    method = 'initialize'
                    params = @{ protocolVersions = @(1) }
                }
                for ($index = 0; $index -lt $Operation.Count; $index++) {
                    $parameters = @{
                        resourceType = 'Microsoft.Windows/Registry'
                        executionType = 'actual'
                    }
                    foreach ($key in $Operation[$index].Keys) {
                        $parameters[$key] = $Operation[$index][$key]
                    }
                    @{
                        jsonrpc = '2.0'
                        id = $index + 1
                        method = 'dsc_invoke'
                        params = $parameters
                    }
                }
                @{ jsonrpc = '2.0'; id = $Operation.Count + 1; method = 'shutdown' }
                @{ jsonrpc = '2.0'; method = 'exit' }
            )
            $responses = $requests | ForEach-Object { $_ | ConvertTo-Json -Depth 20 -Compress } |
                registry jsonrpc 2>$TestDrive\error.log | ConvertFrom-Json -Depth 30
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
            $responses | Should -HaveCount ($Operation.Count + 2)
            for ($index = 0; $index -lt $responses.Count; $index++) {
                $responses[$index].jsonrpc | Should -BeExactly '2.0'
                $responses[$index].id | Should -Be $index
            }
            $responses[0].result.protocolVersion | Should -Be 1
            $responses[-1].PSObject.Properties.Name | Should -Contain 'result'
            $responses[-1].result | Should -BeNullOrEmpty
            $responses | Select-Object -Skip 1 -First $Operation.Count
        }
    }

    BeforeEach {
        $keyName = "DSC-JsonRpc-$([guid]::NewGuid())"
        $keyPath = "HKCU\$keyName"
        $providerPath = "HKCU:\$keyName"
    }

    AfterEach {
        if (Test-Path -LiteralPath $providerPath) {
            Remove-Item -LiteralPath $providerPath -Recurse -Force
        }
    }

    It 'sets, reads, updates, and deletes a value in one process' {
        $responses = @(Invoke-RegistryRpc -Operation @(
            @{ operation = 'set'; properties = @{ keyPath = $keyPath; valueName = 'Value'; valueData = @{ String = 'first' } } }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath; valueName = 'Value' } }
            @{ operation = 'set'; properties = @{ keyPath = $keyPath; valueName = 'Value'; valueData = @{ DWord = 42 } } }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath; valueName = 'Value' } }
            @{ operation = 'delete'; properties = @{ keyPath = $keyPath; valueName = 'Value' } }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath; valueName = 'Value' } }
        ))

        $responses | Where-Object error | Should -BeNullOrEmpty
        foreach ($index in 0, 2, 4) {
            $responses[$index].PSObject.Properties.Name | Should -Contain 'result'
            $responses[$index].result | Should -BeNullOrEmpty
        }
        $responses[1].result.valueData.String | Should -BeExactly 'first'
        $responses[3].result.valueData.DWord | Should -Be 42
        $responses[5].result._exist | Should -BeFalse
        Test-Path -LiteralPath $providerPath | Should -BeTrue
    }

    It 'creates and removes a key using _exist false' {
        $responses = @(Invoke-RegistryRpc -Operation @(
            @{ operation = 'set'; properties = @{ keyPath = "$keyPath\Child" } }
            @{ operation = 'get'; properties = @{ keyPath = "$keyPath\Child" } }
            @{ operation = 'set'; properties = @{ keyPath = $keyPath; _exist = $false } }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath } }
        ))

        $responses | Where-Object error | Should -BeNullOrEmpty
        $responses[1].result.keyPath | Should -BeExactly "$keyPath\Child"
        $responses[1].result.PSObject.Properties.Name | Should -Not -Contain '_exist'
        $responses[3].result._exist | Should -BeFalse
        Test-Path -LiteralPath $providerPath | Should -BeFalse
    }

    It 'previews key creation without changing the registry' {
        $responses = @(Invoke-RegistryRpc -Operation @(
            @{
                operation = 'set'
                executionType = 'whatIf'
                properties = @{ keyPath = "$keyPath\Child"; valueName = 'Value'; valueData = @{ String = 'preview' } }
            }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath } }
        ))

        $responses | Where-Object error | Should -BeNullOrEmpty
        $responses[0].result.valueData.String | Should -BeExactly 'preview'
        $responses[0].result._metadata.whatIf | Should -Not -BeNullOrEmpty
        $responses[1].result._exist | Should -BeFalse
        Test-Path -LiteralPath $providerPath | Should -BeFalse
    }

    It 'previews <Operation> without changing an existing value' -ForEach @(
        @{ Operation = 'set'; Properties = @{ valueData = @{ String = 'preview' } } }
        @{ Operation = 'delete'; Properties = @{} }
        @{ Operation = 'set'; Properties = @{ _exist = $false } }
    ) {
        New-Item -Path $providerPath | Out-Null
        New-ItemProperty -LiteralPath $providerPath -Name 'Value' -Value 'original' | Out-Null
        $desired = @{ keyPath = $keyPath; valueName = 'Value' } + $Properties

        $responses = @(Invoke-RegistryRpc -Operation @(
            @{ operation = $Operation; executionType = 'whatIf'; properties = $desired }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath; valueName = 'Value' } }
        ))

        $responses | Where-Object error | Should -BeNullOrEmpty
        $responses[0].result.keyPath | Should -BeExactly $keyPath
        $responses[1].result.valueData.String | Should -BeExactly 'original'
        Get-ItemPropertyValue -LiteralPath $providerPath -Name 'Value' | Should -BeExactly 'original'
    }

    It 'returns operation errors without corrupting subsequent responses' {
        $responses = @(Invoke-RegistryRpc -Operation @(
            @{ operation = 'get'; properties = @{ keyPath = $keyPath; acl = 'invalid' } }
            @{
                resourceType = 'Microsoft.Windows/RegistryList'
                operation = 'get'
                properties = @{ registryEntries = @(
                    @{ keyPath = $keyPath }
                    @{ keyPath = $keyPath; acl = 'invalid' }
                ) }
            }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath } }
            @{ operation = 'schema' }
        ))

        foreach ($index in 0, 1) {
            $responses[$index].error.code | Should -Be -32000
            $responses[$index].error.message | Should -Match 'Registry ACL'
            $responses[$index].PSObject.Properties.Name | Should -Not -Contain 'result'
        }
        $responses[2].result._exist | Should -BeFalse
        $responses[3].result.properties.keyPath.type | Should -BeExactly 'string'
    }

    It 'uses a separate offline hive for each invocation' {
        $hivePath = Join-Path $TestDrive 'rpc.hiv'
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'test_hives\HKLM.hiv') -Destination $hivePath
        $responses = @(Invoke-RegistryRpc -Operation @(
            @{
                operation = 'get'
                properties = @{ keyPath = 'HKLM\Software\DSCTest'; valueName = 'TestDword'; registryFilePath = $hivePath }
            }
            @{
                operation = 'delete'
                executionType = 'whatIf'
                properties = @{ keyPath = 'HKLM\DSC-Missing'; registryFilePath = $hivePath }
            }
            @{ operation = 'get'; properties = @{ keyPath = $keyPath } }
        ))

        $responses | Where-Object error | Should -BeNullOrEmpty
        $responses[0].result.valueData.DWord | Should -Be 42
        $responses[1].result.keyPath | Should -BeExactly 'HKLM\DSC-Missing'
        $responses[1].result._exist | Should -BeFalse
        $responses[2].result._exist | Should -BeFalse
    }

    It 'inherits list-level offline hives while preserving entry overrides' {
        $defaultHive = Join-Path $TestDrive 'default.hiv'
        $overrideHive = Join-Path $TestDrive 'override.hiv'
        $sourceHive = Join-Path $PSScriptRoot 'test_hives\HKLM.hiv'
        Copy-Item -LiteralPath $sourceHive -Destination $defaultHive
        Copy-Item -LiteralPath $sourceHive -Destination $overrideHive
        $entries = @(
            @{ keyPath = 'HKLM\Software\DSCTest'; valueName = 'TestDword'; valueData = @{ DWord = 10 } }
            @{
                keyPath = 'HKLM\Software\DSCTest'
                valueName = 'TestDword'
                valueData = @{ DWord = 20 }
                registryFilePath = $overrideHive
            }
        )
        $desired = @{ registryEntries = $entries; registryFilePath = $defaultHive }
        $preview = @{
            registryFilePath = $defaultHive
            registryEntries = @(
                @{ keyPath = 'HKLM\Software\DSCTest'; valueName = 'TestDword'; valueData = @{ DWord = 30 } }
                @{
                    keyPath = 'HKLM\Software\DSCTest'
                    valueName = 'TestDword'
                    valueData = @{ DWord = 40 }
                    registryFilePath = $overrideHive
                }
            )
        }

        $responses = @(Invoke-RegistryRpc -Operation @(
            @{ resourceType = 'Microsoft.Windows/RegistryList'; operation = 'set'; properties = $desired }
            @{ resourceType = 'Microsoft.Windows/RegistryList'; operation = 'get'; properties = $desired }
            @{
                resourceType = 'Microsoft.Windows/RegistryList'
                operation = 'set'
                executionType = 'whatIf'
                properties = $preview
            }
            @{ resourceType = 'Microsoft.Windows/RegistryList'; operation = 'get'; properties = $desired }
        ))

        $responses | Where-Object error | Should -BeNullOrEmpty
        $responses[0].PSObject.Properties.Name | Should -Contain 'result'
        $responses[0].result | Should -BeNullOrEmpty
        $responses[1].result.registryEntries.valueData.DWord | Should -Be @(10, 20)
        $responses[2].result.registryEntries.valueData.DWord | Should -Be @(30, 40)
        $responses[3].result.registryEntries.valueData.DWord | Should -Be @(10, 20)
    }
}

Describe 'Registry DSC transport integration with reuse <Reuse>' -Skip:(!$IsWindows) -ForEach @(
    @{ Reuse = 'true' }
    @{ Reuse = 'false' }
) {
    BeforeAll {
        $previousReuse = $env:DSC_RESOURCE_REUSE
        $previousResourcePath = $env:DSC_RESOURCE_PATH
        $env:DSC_RESOURCE_REUSE = $Reuse
        $resourceDirectory = Join-Path $TestDrive 'resources'
        New-Item -Path $resourceDirectory -ItemType Directory | Out-Null
        $manifestPath = Join-Path $PSScriptRoot '..\registry.dsc.manifests.json'
        $manifests = @((Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -AsHashtable).resources |
            Where-Object type -In 'Microsoft.Windows/Registry', 'Microsoft.Windows/RegistryList')
        foreach ($manifest in $manifests) {
            if ($Reuse -eq 'true') {
                # JSON-RPC-only manifests prove these tests can't use the executable fallbacks.
                foreach ($operation in 'get', 'set', 'delete') {
                    if ($manifest.ContainsKey($operation)) {
                        $manifest[$operation].Remove('executable')
                        $manifest[$operation].Remove('args')
                    }
                }
                $manifest.schema.command = @{}
            }
        }
        @{ resources = $manifests } | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath (Join-Path $resourceDirectory 'registry.dsc.manifests.json')
        $env:DSC_RESOURCE_PATH = $resourceDirectory
    }

    BeforeEach {
        $keyName = "DSC-JsonRpc-$([guid]::NewGuid())"
        $keyPath = "HKCU\$keyName"
        $providerPath = "HKCU:\$keyName"
        $properties = @{ keyPath = $keyPath; valueName = 'Value'; valueData = @{ String = 'desired' } }
        $json = $properties | ConvertTo-Json -Compress
    }

    AfterEach {
        if (Test-Path -LiteralPath $providerPath) {
            Remove-Item -LiteralPath $providerPath -Recurse -Force
        }
    }

    AfterAll {
        $env:DSC_RESOURCE_REUSE = $previousReuse
        $env:DSC_RESOURCE_PATH = $previousResourcePath
    }

    It 'supports standalone set, synthetic test, get, and delete' {
        $set = dsc resource set -r Microsoft.Windows/Registry -i $json 2>$TestDrive\error.log |
            ConvertFrom-Json -Depth 20
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $set.afterState.valueData.String | Should -BeExactly 'desired'

        $test = dsc resource test -r Microsoft.Windows/Registry -i $json 2>$TestDrive\error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $test.inDesiredState | Should -BeTrue

        $get = dsc resource get -r Microsoft.Windows/Registry -i $json 2>$TestDrive\error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $get.actualState.valueData.String | Should -BeExactly 'desired'

        $null = dsc resource delete -r Microsoft.Windows/Registry -i $json 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        (Get-Item -LiteralPath $providerPath).Property | Should -Not -Contain 'Value'
    }

    It 'supports configuration what-if and multiple instances' {
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{ name = 'first'; type = 'Microsoft.Windows/Registry'; properties = $properties }
                @{
                    name = 'second'
                    type = 'Microsoft.Windows/Registry'
                    properties = @{ keyPath = $keyPath; valueName = 'Other'; valueData = @{ DWord = 42 } }
                }
            )
        } | ConvertTo-Json -Depth 20 -Compress

        $preview = dsc config set --what-if -i $configuration 2>$TestDrive\error.log | ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $preview.hadErrors | Should -BeFalse
        $preview.results | Should -HaveCount 2
        $preview.results[0].result.afterState.valueData.String | Should -BeExactly 'desired'
        Test-Path -LiteralPath $providerPath | Should -BeFalse

        $set = dsc config set -i $configuration 2>$TestDrive\error.log | ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $set.hadErrors | Should -BeFalse
        $set.results[1].result.afterState.valueData.DWord | Should -Be 42
        Get-ItemPropertyValue -LiteralPath $providerPath -Name 'Value' | Should -BeExactly 'desired'
        Get-ItemPropertyValue -LiteralPath $providerPath -Name 'Other' | Should -Be 42
    }

    It 'supports delete what-if without removing the key' {
        New-Item -Path $providerPath | Out-Null
        $inputJson = @{ keyPath = $keyPath } | ConvertTo-Json -Compress
        $preview = dsc resource delete --what-if -r Microsoft.Windows/Registry -i $inputJson 2>$TestDrive\error.log |
            ConvertFrom-Json -Depth 20

        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $preview._metadata.whatIf | Should -Not -BeNullOrEmpty
        Test-Path -LiteralPath $providerPath | Should -BeTrue
    }

    It 'supports RegistryList set, get, synthetic test, and per-entry deletion' {
        $entries = @(
            $properties
            @{ keyPath = $keyPath; valueName = 'Other'; valueData = @{ DWord = 42 } }
        )
        $inputJson = @{ registryEntries = $entries } | ConvertTo-Json -Depth 10 -Compress
        $set = dsc resource set -r Microsoft.Windows/RegistryList -i $inputJson 2>$TestDrive\error.log |
            ConvertFrom-Json -Depth 20
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $set.afterState.registryEntries | Should -HaveCount 2
        $set.afterState.registryEntries[0].valueData.String | Should -BeExactly 'desired'
        $set.afterState.registryEntries[1].valueData.DWord | Should -Be 42

        $test = dsc resource test -r Microsoft.Windows/RegistryList -i $inputJson 2>$TestDrive\error.log |
            ConvertFrom-Json -Depth 20
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $test.inDesiredState | Should -BeTrue

        $entries[0]._exist = $false
        $deleteJson = @{ registryEntries = $entries } | ConvertTo-Json -Depth 10 -Compress
        $null = dsc resource set -r Microsoft.Windows/RegistryList -i $deleteJson 2>$TestDrive\error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)

        $get = dsc resource get -r Microsoft.Windows/RegistryList -i $inputJson 2>$TestDrive\error.log |
            ConvertFrom-Json -Depth 20
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $get.actualState.registryEntries[0]._exist | Should -BeFalse
        $get.actualState.registryEntries[1].valueData.DWord | Should -Be 42
    }

    It 'supports RegistryList configuration previews and mixed entry changes' {
        New-Item -Path $providerPath | Out-Null
        New-ItemProperty -LiteralPath $providerPath -Name 'Value' -Value 'original' | Out-Null
        New-ItemProperty -LiteralPath $providerPath -Name 'Other' -Value 42 | Out-Null
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'list'
                    type = 'Microsoft.Windows/RegistryList'
                    properties = @{
                        registryEntries = @(
                            $properties
                            @{ keyPath = $keyPath; valueName = 'Other'; _exist = $false }
                            @{ keyPath = "$keyPath\Child" }
                        )
                    }
                }
            )
        } | ConvertTo-Json -Depth 20 -Compress

        $preview = dsc config set --what-if -i $configuration 2>$TestDrive\error.log |
            ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $preview.hadErrors | Should -BeFalse
        $preview.results[0].result.afterState.registryEntries | Should -HaveCount 3
        $preview.results[0].result.afterState.registryEntries[0].valueData.String | Should -BeExactly 'desired'
        $preview.results[0].result.afterState.registryEntries[1]._metadata.whatIf | Should -Not -BeNullOrEmpty
        Get-ItemPropertyValue -LiteralPath $providerPath -Name 'Value' | Should -BeExactly 'original'
        Get-ItemPropertyValue -LiteralPath $providerPath -Name 'Other' | Should -Be 42
        Test-Path -LiteralPath "$providerPath\Child" | Should -BeFalse

        $set = dsc config set -i $configuration 2>$TestDrive\error.log | ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $set.hadErrors | Should -BeFalse
        Get-ItemPropertyValue -LiteralPath $providerPath -Name 'Value' | Should -BeExactly 'desired'
        (Get-Item -LiteralPath $providerPath).Property | Should -Not -Contain 'Other'
        Test-Path -LiteralPath "$providerPath\Child" | Should -BeTrue
    }
}
