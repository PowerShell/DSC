# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'JSON-RPC extensions and reverse secret requests' {
    BeforeAll {
        $fixtureRoot = (Resolve-Path "$PSScriptRoot/../../../extensions/test/jsonrpc").Path
        $discoverPath = Join-Path $fixtureRoot discover
        $secretPath = Join-Path $fixtureRoot secret

        function Set-RpcSettings {
            $settings | ConvertTo-Json -Depth 30 | Set-Content -LiteralPath $env:DSC_TEST_EXTENSION_RPC_SETTINGS
        }

        function Get-ExtensionRpcEvent {
            if (Test-Path -LiteralPath $env:DSC_TEST_EXTENSION_RPC_LOG) {
                @(Get-Content -LiteralPath $env:DSC_TEST_EXTENSION_RPC_LOG | ConvertFrom-Json)
            }
        }

        function Get-SecretDiscoveredResource {
            $output = @(dsc -l trace resource list 'Test/SecretDiscovered' 2>$TestDrive/error.log)
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive/error.log -Raw)
            $output | ConvertFrom-Json
        }

        function New-NetworkDiscoverFixture {
            $manifest = Get-Content (Join-Path $discoverPath discover.dsc.extension.json) -Raw |
                ConvertFrom-Json -AsHashtable
            $manifest.type = 'Test/ANetworkDiscover'
            $manifest.discover.network = $true
            $manifest.discover.jsonrpc.args[-1] = Join-Path $discoverPath discover.ps1
            $manifest | ConvertTo-Json -Depth 20 |
                Set-Content (Join-Path $testDirectory network.dsc.extension.json)
            $env:DSC_RESTRICTED_PATH = @($testDirectory, $secretPath, $PSHOME) -join [IO.Path]::PathSeparator
        }

        function New-LocalDiscoveryManifest {
            param([string]$Type = 'Test/SecretDiscovered', [string]$Version = '0.1.0')
            @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                type = $Type
                version = $Version
                get = @{
                    executable = 'pwsh'
                    args = @('-NoProfile', '-NonInteractive', '-Command', '''{"installed":true}''')
                }
                schema = @{ embedded = @{ type = 'object'; properties = @{ installed = @{ type = 'boolean' } } } }
            }
        }

        function New-CommandDiscoverFixture {
            param([string]$Type, [hashtable]$Manifest, [bool]$Network = $false, [switch]$Fail)
            $prefix = Join-Path $testDirectory $Type.Replace('/', '-')
            @{ manifestContent = $Manifest } | ConvertTo-Json -Depth 20 -Compress |
                Set-Content "$prefix.results.json"
            $escapedPrefix = $prefix.Replace("'", "''")
            $command = "Set-Content -LiteralPath '$escapedPrefix.called' -Value called; "
            $command += if ($Fail) { 'exit 1' } else { "Get-Content -LiteralPath '$escapedPrefix.results.json'" }
            $discover = @{ executable = 'pwsh'; args = @('-NoProfile', '-NonInteractive', '-Command', $command) }
            if ($Network) { $discover.network = $true }
            @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json'
                type = $Type
                version = '0.1.0'
                discover = $discover
            } | ConvertTo-Json -Depth 20 | Set-Content "$prefix.dsc.extension.json"
        }

        function New-RegistryAdapterFixture {
            $listed = @{
                type = 'Test/SecretDiscovered'
                kind = 'resource'
                version = '0.1.0'
                capabilities = @('get', 'set')
                requireAdapter = 'Test/InstalledRegistryAdapter'
                description = 'Installed adapter-list result'
            }
            $listed | ConvertTo-Json -Compress | Set-Content (Join-Path $testDirectory listed.json)
            @'
param([string]$Operation)
Add-Content -LiteralPath (Join-Path $PSScriptRoot adapter-calls.log) -Value $Operation
if ($Operation -eq 'list') {
    Get-Content -LiteralPath (Join-Path $PSScriptRoot listed.json)
} else {
    '{"installed":true}'
}
'@ | Set-Content (Join-Path $testDirectory adapter.ps1)
            $arguments = @('-NoProfile', '-NonInteractive', '-File', (Join-Path $testDirectory adapter.ps1))
            @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                type = 'Test/InstalledRegistryAdapter'
                version = '0.1.0'
                kind = 'adapter'
                get = @{ executable = 'pwsh'; args = $arguments + @('-Operation', 'get') }
                set = @{
                    executable = 'pwsh'; args = $arguments + @('-Operation', 'set')
                    implementsPretest = $true; return = 'state'
                }
                adapter = @{
                    inputKind = 'single'
                    list = @{ executable = 'pwsh'; args = $arguments + @('-Operation', 'list') }
                }
                schema = @{ embedded = @{ type = 'object' } }
            } | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $testDirectory adapter.dsc.resource.json)
            $env:PATH = @($testDirectory, $env:PATH) -join [IO.Path]::PathSeparator
            $env:DSC_RESTRICTED_PATH = @($testDirectory, $discoverPath, $secretPath, $PSHOME) -join [IO.Path]::PathSeparator
            $settings.registry = 'oci-test'
            $settings.requests = @()
            $settings.manifest = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/adapted/manifest.json'
                type = 'Test/SecretDiscovered'
                version = '0.1.0'
                kind = 'resource'
                capabilities = @('get', 'set')
                requireAdapter = 'Test/InstalledRegistryAdapter'
                content = @{ remote = $true }
                schema = @{ embedded = @{ type = 'object' } }
            }
            Set-RpcSettings
        }
    }

    BeforeEach {
        $previousPath = $env:PATH
        $previousHome = $env:HOME
        $previousLocalAppData = $env:LOCALAPPDATA
        $previousProgramData = $env:ProgramData
        $previousResourcePath = $env:DSC_RESOURCE_PATH
        $previousRestrictedPath = $env:DSC_RESTRICTED_PATH
        $previousLog = $env:DSC_TEST_EXTENSION_RPC_LOG
        $previousSettings = $env:DSC_TEST_EXTENSION_RPC_SETTINGS
        $previousFault = $env:DSC_TEST_SECRET_RPC_FAULT
        $env:PATH = @($discoverPath, $secretPath, $previousPath) -join [IO.Path]::PathSeparator
        $env:DSC_RESOURCE_PATH = $null
        $env:DSC_RESTRICTED_PATH = $null
        $env:DSC_TEST_EXTENSION_RPC_LOG = Join-Path $TestDrive "$([guid]::NewGuid()).jsonl"
        $env:DSC_TEST_EXTENSION_RPC_SETTINGS = Join-Path $TestDrive "$([guid]::NewGuid()).json"
        $env:DSC_TEST_SECRET_RPC_FAULT = $null
        $testDirectory = (New-Item -ItemType Directory -Path "$TestDrive/$([guid]::NewGuid())").FullName
        $env:HOME = $testDirectory
        $env:LOCALAPPDATA = $testDirectory
        $env:ProgramData = $testDirectory
        $secretName = "runtime-$([guid]::NewGuid().ToString('N'))"
        $stringSecret = @{ secureString = 'test-runtime-token' }
        $objectSecret = @{ secureObject = @{ credential = 'test-runtime-password' } }
        $settings = @{
            requests = @(
                @{ name = $secretName; expected = $stringSecret }
                @{ name = 'object-secret'; vault = 'test-vault'; expected = $objectSecret }
            )
            secrets = @{
                "/$secretName" = $stringSecret
                'test-vault/object-secret' = $objectSecret
            }
        }
        Set-RpcSettings
    }

    AfterEach {
        $processIds = (Get-ExtensionRpcEvent).pid | Select-Object -Unique
        $env:PATH = $previousPath
        $env:HOME = $previousHome
        $env:LOCALAPPDATA = $previousLocalAppData
        $env:ProgramData = $previousProgramData
        $env:DSC_RESOURCE_PATH = $previousResourcePath
        $env:DSC_RESTRICTED_PATH = $previousRestrictedPath
        $env:DSC_TEST_EXTENSION_RPC_LOG = $previousLog
        $env:DSC_TEST_EXTENSION_RPC_SETTINGS = $previousSettings
        $env:DSC_TEST_SECRET_RPC_FAULT = $previousFault
        foreach ($processId in $processIds) {
            Get-Process -Id $processId -ErrorAction SilentlyContinue | Should -BeNullOrEmpty
        }
    }

    It 'retrieves settings-selected string and object secrets using numeric IDs <NumericIds>' -ForEach @(
        @{ NumericIds = $false }
        @{ NumericIds = $true }
    ) {
        $settings.numericIds = $NumericIds
        Set-RpcSettings
        (Get-SecretDiscoveredResource).type | Should -BeExactly 'Test/SecretDiscovered'
        $events = @(Get-ExtensionRpcEvent)
        $calls = @($events | Where-Object { $_.role -eq 'secret' -and $_.event -eq 'dsc_invoke' })
        $calls | Should -HaveCount 2
        $calls[0].details.name | Should -BeExactly $secretName
        $calls[1].details.vault | Should -BeExactly 'test-vault'
        @($events | Where-Object event -EQ callback) | Should -HaveCount 2
        @($events | Where-Object event -EQ initialize) | Should -HaveCount 3
        @($events | Where-Object event -EQ shutdown) | Should -HaveCount 3
        @($events | Where-Object event -EQ exit) | Should -HaveCount 3
        $diagnostics = Get-Content -LiteralPath $TestDrive/error.log -Raw
        $diagnostics | Should -Not -Match 'test-runtime-token|test-runtime-password'
        (Get-Content -LiteralPath $env:DSC_TEST_EXTENSION_RPC_LOG -Raw) |
            Should -Not -Match 'test-runtime-token|test-runtime-password'
    }

    It 'uses the authenticated discovered resource in a configuration' {
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(@{ name = 'authenticated'; type = 'Test/SecretDiscovered' })
        }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 10) |
            ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0
        $result.hadErrors | Should -BeFalse
        $result.results[0].result.actualState.authenticated | Should -BeTrue
    }

    It 'skips network discovery for a locally satisfied <Operation> request' -ForEach @(
        @{ Operation = 'get' }
        @{ Operation = 'schema' }
    ) {
        New-NetworkDiscoverFixture
        New-LocalDiscoveryManifest | ConvertTo-Json -Depth 20 |
            Set-Content (Join-Path $testDirectory local.dsc.resource.json)
        $result = dsc resource $Operation -r Test/SecretDiscovered 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        if ($Operation -eq 'get') { $result.actualState.installed | Should -BeTrue }
        else { $result.properties.installed.type | Should -BeExactly 'boolean' }
        @(Get-ExtensionRpcEvent) | Should -HaveCount 0
    }

    It 'tries non-network Discover extensions before an alphabetically earlier network extension' {
        New-NetworkDiscoverFixture
        New-CommandDiscoverFixture -Type 'Test/ZLocalDiscover' -Manifest (New-LocalDiscoveryManifest)
        $result = dsc resource get -r Test/SecretDiscovered 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result.actualState.installed | Should -BeTrue
        Join-Path $testDirectory 'Test-ZLocalDiscover.called' | Should -Exist
        @(Get-ExtensionRpcEvent) | Should -HaveCount 0
    }

    It 'tries installed adapters before network discovery with adapter pinned <Pinned>' -ForEach @(
        @{ Pinned = $false }
        @{ Pinned = $true }
    ) {
        New-RegistryAdapterFixture
        New-NetworkDiscoverFixture
        $instance = @{ name = 'installed'; type = 'Test/SecretDiscovered' }
        if ($Pinned) { $instance.directives = @{ requireAdapter = 'Test/InstalledRegistryAdapter' } }
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @($instance)
        }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) 2>$TestDrive/error.log |
            ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result.results[0].result.actualState.installed | Should -BeTrue
        @(Get-ExtensionRpcEvent) | Should -HaveCount 0
        @(Get-Content (Join-Path $testDirectory adapter-calls.log)) | Should -Be @('list', 'get')
    }

    It 'falls back to network discovery for <Reason> and skips later network extensions' -ForEach @(
        @{ Reason = 'missing type' }
        @{ Reason = 'unavailable version' }
    ) {
        New-NetworkDiscoverFixture
        New-CommandDiscoverFixture -Type 'Test/ZNetworkDiscover' -Network $true -Manifest (New-LocalDiscoveryManifest)
        $settings.requests = @()
        $versionArguments = @()
        if ($Reason -eq 'unavailable version') {
            New-LocalDiscoveryManifest | ConvertTo-Json -Depth 20 |
                Set-Content (Join-Path $testDirectory local.dsc.resource.json)
            $settings.manifest = New-LocalDiscoveryManifest -Version '0.2.0'
            $versionArguments = @('--version', '=0.2.0')
        }
        Set-RpcSettings
        $result = dsc resource get -r Test/SecretDiscovered @versionArguments 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        if ($Reason -eq 'missing type') { $result.actualState.authenticated | Should -BeTrue }
        else { $result.actualState.installed | Should -BeTrue }
        @((Get-ExtensionRpcEvent) | Where-Object { $_.role -eq 'discover' -and $_.event -eq 'dsc_invoke' }) |
            Should -HaveCount 1
        Join-Path $testDirectory 'Test-ZNetworkDiscover.called' | Should -Not -Exist
    }

    It 'preserves locally selected resources when network discovery resolves another configuration resource' {
        New-NetworkDiscoverFixture
        New-LocalDiscoveryManifest -Type 'Test/Local' | ConvertTo-Json -Depth 20 |
            Set-Content (Join-Path $testDirectory local.dsc.resource.json)
        $settings.requests = @()
        $settings.additionalManifest = New-LocalDiscoveryManifest -Type 'Test/Local' -Version '9.0.0'
        $settings.additionalManifest.get.args[-1] = '''{"remote":true}'''
        Set-RpcSettings
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{ name = 'remote'; type = 'Test/SecretDiscovered' }
                @{ name = 'local'; type = 'Test/Local' }
            )
        }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) 2>$TestDrive/error.log |
            ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result.results[0].result.actualState.authenticated | Should -BeTrue
        $result.results[1].result.actualState.installed | Should -BeTrue
    }

    It 'continues network discovery when an earlier result does not satisfy the requested version' {
        New-NetworkDiscoverFixture
        $settings.requests = @()
        Set-RpcSettings
        New-CommandDiscoverFixture -Type 'Test/ZNetworkDiscover' -Network $true `
            -Manifest (New-LocalDiscoveryManifest -Version '0.2.0')
        $result = dsc resource get -r Test/SecretDiscovered --version '=0.2.0' 2>$TestDrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result.actualState.installed | Should -BeTrue
        @((Get-ExtensionRpcEvent) | Where-Object { $_.role -eq 'discover' -and $_.event -eq 'dsc_invoke' }) |
            Should -HaveCount 1
        Join-Path $testDirectory 'Test-ZNetworkDiscover.called' | Should -Exist
    }

    It 'includes network adapted metadata in adapter-filtered lists' {
        New-RegistryAdapterFixture
        New-NetworkDiscoverFixture
        $settings.manifest.type = 'Test/RemoteOnly'
        Set-RpcSettings
        $resources = @(dsc resource list -a Test/InstalledRegistryAdapter 2>$TestDrive/error.log | ConvertFrom-Json)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources | Should -HaveCount 2
        ($resources | Where-Object type -EQ 'Test/RemoteOnly').registry | Should -BeExactly 'oci-test'
        ($resources | Where-Object type -EQ 'Test/SecretDiscovered').registry | Should -BeExactly 'local'
    }

    It 'filters network list results by the requested type' {
        New-NetworkDiscoverFixture
        $settings.requests = @()
        $settings.additionalManifest = New-LocalDiscoveryManifest -Type 'Test/OtherRemote'
        Set-RpcSettings
        $resources = @(dsc resource list Test/SecretDiscovered 2>$TestDrive/error.log | ConvertFrom-Json)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources | Should -HaveCount 1
        $resources[0].type | Should -BeExactly 'Test/SecretDiscovered'
    }

    It 'reads network adapted schemas without requiring an installed adapter' {
        New-RegistryAdapterFixture
        Remove-Item -LiteralPath (Join-Path $testDirectory adapter.dsc.resource.json)
        New-NetworkDiscoverFixture
        Set-RpcSettings
        $schema = dsc resource schema -r Test/SecretDiscovered 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $schema.type | Should -BeExactly 'object'
        Join-Path $testDirectory adapter-calls.log | Should -Not -Exist
    }

    It 'refreshes network results for each during-deployment lookup' {
        New-NetworkDiscoverFixture
        $settings.requests = @()
        Set-RpcSettings
        $update = New-LocalDiscoveryManifest -Type 'Test/UpdateNetworkResults'
        $escapedSettings = $env:DSC_TEST_EXTENSION_RPC_SETTINGS.Replace("'", "''")
        $update.get.args[-1] = "Set-Content -LiteralPath '$escapedSettings' -Value '{`"requests`":[],`"results`":[]}'; '{`"updated`":true}'"
        $update | ConvertTo-Json -Depth 20 | Set-Content (Join-Path $testDirectory update.dsc.resource.json)
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            directives = @{ resourceDiscovery = 'duringDeployment' }
            resources = @(
                @{ name = 'before'; type = 'Test/SecretDiscovered' }
                @{
                    name = 'update'; type = 'Test/UpdateNetworkResults'
                    dependsOn = @("[resourceId('Test/SecretDiscovered', 'before')]")
                }
                @{
                    name = 'after'; type = 'Test/SecretDiscovered'
                    dependsOn = @("[resourceId('Test/UpdateNetworkResults', 'update')]")
                }
            )
        }
        dsc config get -i ($configuration | ConvertTo-Json -Depth 20) 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive/error.log | Should -Match 'Resource not found: Test/SecretDiscovered'
        @((Get-ExtensionRpcEvent) | Where-Object { $_.role -eq 'discover' -and $_.event -eq 'dsc_invoke' }) |
            Should -HaveCount 2
    }

    It 'continues after a failed network extension and lists all network extensions after local discovery' {
        New-NetworkDiscoverFixture
        $settings.requests = @()
        Set-RpcSettings
        New-CommandDiscoverFixture -Type 'Test/0FailedNetwork' -Network $true -Fail -Manifest (New-LocalDiscoveryManifest)
        New-CommandDiscoverFixture -Type 'Test/ZLocalDiscover' -Manifest (New-LocalDiscoveryManifest -Type 'Test/Local')
        New-CommandDiscoverFixture -Type 'Test/ZNetworkDiscover' -Network $true -Manifest (New-LocalDiscoveryManifest -Type 'Test/OtherRemote')
        $resources = @(dsc -l debug resource list 'Test/*' 2>$TestDrive/error.log | ConvertFrom-Json)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources.type | Should -Contain 'Test/Local'
        $resources.type | Should -Contain 'Test/SecretDiscovered'
        $resources.type | Should -Contain 'Test/OtherRemote'
        $diagnostics = Get-Content -Raw $TestDrive/error.log
        $diagnostics | Should -Match 'Test/0FailedNetwork'
        $diagnostics.IndexOf("Calling extension 'Test/ZLocalDiscover'") |
            Should -BeLessThan $diagnostics.IndexOf("Calling extension 'Test/ANetworkDiscover'")
    }

    It 'reports the active Discover extension during <Kind> discovery' -ForEach @(
        @{ Kind = 'resource' }
        @{ Kind = 'action' }
    ) {
        $settings.requests = @()
        Set-RpcSettings
        dsc --trace-format json --progress-format json $Kind list 'Test/*' 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $progress = @(Get-Content $TestDrive/error.log | ConvertFrom-Json | Where-Object {
            $_.activity -eq "Calling extension 'Test/JsonRpcDiscover' to discover resources"
        })
        @($progress | Where-Object completedItems -EQ 0) | Should -HaveCount 1
        $progress[0].totalItems | Should -Be 1
        $progress[0].id | Should -Not -BeNullOrEmpty
    }

    It 'preserves the registry friendly name from a Discover result' {
        $settings.registry = 'oci-test'
        Set-RpcSettings
        (Get-SecretDiscoveredResource).registry | Should -BeExactly 'oci-test'
    }

    It 'lists and invokes the installed local resource instead of its remote duplicate' {
        $localManifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/SecretDiscovered'
            version = '0.1.0'
            get = @{
                executable = 'pwsh'
                args = @('-NoProfile', '-NonInteractive', '-Command', '''{"installed":true}''')
            }
            schema = @{ embedded = @{ type = 'object'; properties = @{ installed = @{ type = 'boolean' } } } }
        }
        $manifestPath = Join-Path $testDirectory 'installed.dsc.resource.json'
        $localManifest | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $manifestPath
        $env:PATH = @($testDirectory, $env:PATH) -join [IO.Path]::PathSeparator
        $settings.registry = 'oci-test'
        $settings.requests = @()
        Set-RpcSettings

        $listed = @(Get-SecretDiscoveredResource)
        $listed | Should -HaveCount 1
        $listed[0].registry | Should -BeExactly 'local'
        $listed[0].path | Should -BeExactly $manifestPath
        $result = dsc resource get -r Test/SecretDiscovered 2>$TestDrive\error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $result.actualState.installed | Should -BeTrue
    }

    It 'prefers installed adapter-list resources for <Operation> with adapter pinned <Pinned> and remote version <RemoteVersion>' -ForEach @(
        @{ Operation = 'get'; Pinned = $false; RemoteVersion = '0.1.0' }
        @{ Operation = 'get'; Pinned = $true; RemoteVersion = '0.1.0' }
        @{ Operation = 'set'; Pinned = $false; RemoteVersion = '0.1.0' }
        @{ Operation = 'set'; Pinned = $true; RemoteVersion = '0.1.0' }
        @{ Operation = 'get'; Pinned = $false; RemoteVersion = '0.2.0' }
    ) {
        New-RegistryAdapterFixture
        $settings.manifest.version = $RemoteVersion
        Set-RpcSettings
        $instance = @{
            name = 'installed'
            type = 'Test/SecretDiscovered'
            requireVersion = '=0.1.0'
            properties = @{ installed = $true }
        }
        if ($Pinned) {
            $instance.directives = @{ requireAdapter = 'Test/InstalledRegistryAdapter' }
        }
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @($instance)
        }
        $result = dsc config $Operation -i ($configuration | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log |
            ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result.hadErrors | Should -BeFalse
        if ($Operation -eq 'get') {
            $result.results[0].result.actualState.installed | Should -BeTrue
        } else {
            $result.results[0].result.afterState.installed | Should -BeTrue
        }
    }

    It 'uses a different installed adapter for <Operation> when the advertised adapter is installed <AdvertisedInstalled>' -ForEach @(
        @{ Operation = 'get'; AdvertisedInstalled = $false }
        @{ Operation = 'get'; AdvertisedInstalled = $true }
        @{ Operation = 'set'; AdvertisedInstalled = $false }
        @{ Operation = 'set'; AdvertisedInstalled = $true }
    ) {
        New-RegistryAdapterFixture
        $settings.manifest.requireAdapter = 'Test/AdvertisedRegistryAdapter'
        $marker = Join-Path $testDirectory advertised-adapter-called
        if ($AdvertisedInstalled) {
            $escapedMarker = $marker.Replace("'", "''")
            @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                type = 'Test/AdvertisedRegistryAdapter'
                version = '0.1.0'
                kind = 'adapter'
                adapter = @{
                    inputKind = 'single'
                    list = @{
                        executable = 'pwsh'
                        args = @('-NoProfile', '-NonInteractive', '-Command',
                            "Set-Content -LiteralPath '$escapedMarker' -Value listed")
                    }
                }
            } | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $testDirectory advertised.dsc.resource.json)
        }
        Set-RpcSettings
        $result = dsc resource $Operation -r Test/SecretDiscovered --version '=0.1.0' -i '{"installed":true}' 2>$TestDrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        if ($Operation -eq 'get') {
            $result.actualState.installed | Should -BeTrue
        } else {
            $result.afterState.installed | Should -BeTrue
        }
        if ($AdvertisedInstalled) {
            $marker | Should -Exist
        } else {
            $marker | Should -Not -Exist
        }
    }

    It 'retains installed candidates for different explicit adapter requirements in one configuration' {
        New-RegistryAdapterFixture
        $otherAdapter = 'Test/OtherInstalledAdapter'
        $listed = Get-Content -Raw (Join-Path $testDirectory listed.json) | ConvertFrom-Json
        $listed.requireAdapter = $otherAdapter
        $listed | ConvertTo-Json -Compress | Set-Content (Join-Path $testDirectory other-listed.json)
        $manifest = Get-Content -Raw (Join-Path $testDirectory adapter.dsc.resource.json) | ConvertFrom-Json -AsHashtable
        $manifest.type = $otherAdapter
        $escapedPath = (Join-Path $testDirectory other-listed.json).Replace("'", "''")
        $manifest.adapter.list.args = @('-NoProfile', '-NonInteractive', '-Command', "Get-Content -LiteralPath '$escapedPath'")
        $manifest | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $testDirectory other-adapter.dsc.resource.json)
        $settings.additionalManifest = $settings.manifest.Clone()
        $settings.additionalManifest.requireAdapter = $otherAdapter
        Set-RpcSettings
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'first'
                    type = 'Test/SecretDiscovered'
                    directives = @{ requireAdapter = 'Test/InstalledRegistryAdapter' }
                }
                @{
                    name = 'second'
                    type = 'Test/SecretDiscovered'
                    directives = @{ requireAdapter = $otherAdapter }
                }
            )
        }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log |
            ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $result.hadErrors | Should -BeFalse
        $result.results | Should -HaveCount 2
        $result.results[0].result.actualState.installed | Should -BeTrue
        $result.results[1].result.actualState.installed | Should -BeTrue
    }

    It 'refreshes adapted registry candidates after an implementation is removed during deployment' {
        New-RegistryAdapterFixture
        $escapedPath = (Join-Path $testDirectory listed.json).Replace("'", "''")
        @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/UpdateAdapterList'
            version = '0.1.0'
            get = @{
                executable = 'pwsh'
                args = @('-NoProfile', '-NonInteractive', '-Command',
                    "Set-Content -LiteralPath '$escapedPath' -Value '' -NoNewline; '{`"updated`":true}'")
            }
            schema = @{ embedded = @{ type = 'object' } }
        } | ConvertTo-Json -Depth 10 | Set-Content (Join-Path $testDirectory update.dsc.resource.json)
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            directives = @{ resourceDiscovery = 'duringDeployment' }
            resources = @(
                @{ name = 'before'; type = 'Test/SecretDiscovered' }
                @{
                    name = 'remove'
                    type = 'Test/UpdateAdapterList'
                    dependsOn = @("[resourceId('Test/SecretDiscovered', 'before')]")
                }
                @{
                    name = 'after'
                    type = 'Test/SecretDiscovered'
                    dependsOn = @("[resourceId('Test/UpdateAdapterList', 'remove')]")
                }
            )
        }
        dsc config get -i ($configuration | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log | Out-Null
        $LASTEXITCODE | Should -Be 2
        Get-Content -Raw $TestDrive/error.log | Should -Match "Resource 'Test/SecretDiscovered' from registry 'oci-test' is not installed"
        Get-Content -Raw (Join-Path $testDirectory listed.json) | Should -BeNullOrEmpty
        $calls = @(Get-Content (Join-Path $testDirectory adapter-calls.log))
        @($calls | Where-Object { $_ -eq 'list' }) | Should -HaveCount 2
        @($calls | Where-Object { $_ -eq 'get' }) | Should -HaveCount 1
    }

    It 'does not search other installed adapters when the remote adapter is explicitly required' {
        New-RegistryAdapterFixture
        $settings.manifest.requireAdapter = 'Test/AdvertisedRegistryAdapter'
        Set-RpcSettings
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(@{
                name = 'pinned'
                type = 'Test/SecretDiscovered'
                directives = @{ requireAdapter = 'Test/AdvertisedRegistryAdapter' }
            })
        }
        dsc config get -i ($configuration | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        Get-Content -Raw $TestDrive/error.log | Should -Match "Resource 'Test/SecretDiscovered' from registry 'oci-test' is not installed"
        Join-Path $testDirectory adapter-calls.log | Should -Not -Exist
    }

    It 'reads remote embedded schemas with network <Network> without calling an installed adapter that provides the resource <ProvidesResource>' -ForEach @(
        @{ Network = $false; ProvidesResource = $false }
        @{ Network = $false; ProvidesResource = $true }
        @{ Network = $true; ProvidesResource = $false }
        @{ Network = $true; ProvidesResource = $true }
    ) {
        New-RegistryAdapterFixture
        if ($Network) { New-NetworkDiscoverFixture }
        if (-not $ProvidesResource) {
            $listed = Get-Content -Raw (Join-Path $testDirectory listed.json) | ConvertFrom-Json
            $listed.type = 'Test/OtherResource'
            $listed | ConvertTo-Json -Compress | Set-Content (Join-Path $testDirectory listed.json)
        }
        $settings.manifest.schema = @{
            embedded = @{ type = 'object'; properties = @{ metadataOnly = @{ type = 'boolean' } } }
        }
        Set-RpcSettings
        $schema = dsc resource schema -r Test/SecretDiscovered --version '=0.1.0' 2>$TestDrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $schema.properties.metadataOnly.type | Should -BeExactly 'boolean'
        Join-Path $testDirectory adapter-calls.log | Should -Not -Exist
    }

    It 'still discovers installed adapted schemas when no matching resource metadata is available with network <Network>' -ForEach @(
        @{ Network = $false }
        @{ Network = $true }
    ) {
        New-RegistryAdapterFixture
        if ($Network) { New-NetworkDiscoverFixture }
        $settings.manifest.type = 'Test/OtherMetadata'
        Set-RpcSettings
        $schema = dsc resource schema -r Test/SecretDiscovered --version '=0.1.0' 2>$TestDrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $schema.type | Should -BeExactly 'object'
        Get-Content (Join-Path $testDirectory adapter-calls.log) | Should -Contain 'list'
    }

    It 'includes remote versions in adapter-filtered lists but keeps installed duplicates first' {
        New-RegistryAdapterFixture
        $settings.additionalManifest = $settings.manifest.Clone()
        $settings.additionalManifest.version = '0.2.0'
        Set-RpcSettings
        $resources = @(dsc resource list Test/SecretDiscovered --adapter Test/InstalledRegistryAdapter 2>$TestDrive/error.log |
            ConvertFrom-Json)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources | Should -HaveCount 2
        $installed = @($resources | Where-Object version -EQ '0.1.0')
        $installed | Should -HaveCount 1
        $installed[0].registry | Should -BeExactly 'local'
        $installed[0].description | Should -BeExactly 'Installed adapter-list result'
        $remote = @($resources | Where-Object version -EQ '0.2.0')
        $remote | Should -HaveCount 1
        $remote[0].registry | Should -BeExactly 'oci-test'
        $remote[0].requireAdapter | Should -BeExactly 'Test/InstalledRegistryAdapter'
        $remote[0].path | Should -BeNullOrEmpty
    }

    It 'lists remote adapted metadata without executing its uninstalled adapter' {
        $marker = Join-Path $testDirectory remote-adapter-executed
        $escapedMarker = $marker.Replace("'", "''")
        $settings.registry = 'oci-test'
        $settings.requests = @()
        $settings.manifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/RemoteAdapter'
            version = '0.1.0'
            kind = 'adapter'
            adapter = @{
                inputKind = 'single'
                list = @{
                    executable = 'pwsh'
                    args = @('-NoProfile', '-NonInteractive', '-Command',
                        "Set-Content -LiteralPath '$escapedMarker' -Value executed")
                }
            }
            schema = @{ embedded = @{ type = 'object' } }
        }
        $settings.additionalManifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/adapted/manifest.json'
            type = 'Test/SecretDiscovered'
            version = '0.1.0'
            kind = 'resource'
            capabilities = @('get')
            requireAdapter = 'Test/RemoteAdapter'
            content = @{ remote = $true }
            schema = @{ embedded = @{ type = 'object' } }
        }
        Set-RpcSettings
        $resources = @(dsc resource list Test/SecretDiscovered --adapter 'Test/Remote*' 2>$TestDrive/error.log |
            ConvertFrom-Json)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources | Should -HaveCount 1
        $resources[0].registry | Should -BeExactly 'oci-test'
        $resources[0].requireAdapter | Should -BeExactly 'Test/RemoteAdapter'
        $marker | Should -Not -Exist
    }

    It 'rejects remote <Shape> manifests without executing schema commands' -ForEach @(
        @{ Shape = 'action' }
        @{ Shape = 'wrong-schema' }
    ) {
        $marker = Join-Path $testDirectory 'schema-command-executed'
        $manifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/action/manifest.json'
            type = 'Test/SecretDiscovered'
            version = '0.1.0'
        }
        switch ($Shape) {
            action {
                $escapedMarker = $marker.Replace("'", "''")
                $manifest.invoke = @{
                    executable = 'pwsh'
                    inputSchema = @{
                        command = @{
                            executable = 'pwsh'
                            args = @('-NoProfile', '-NonInteractive', '-Command',
                                "Set-Content -LiteralPath '$escapedMarker' -Value executed; '{`"type`":`"object`"}'")
                        }
                    }
                }
            }
            wrong-schema {
                $manifest.get = @{ executable = 'pwsh' }
                $manifest.schema = @{ embedded = @{ type = 'object' } }
            }
        }
        $settings.registry = 'oci-test'
        $settings.requests = @()
        $settings.manifest = $manifest
        Set-RpcSettings
        @(Get-SecretDiscoveredResource) | Should -HaveCount 0
        $marker | Should -Not -Exist
        Get-Content -LiteralPath $TestDrive\error.log -Raw | Should -Match "Registry 'oci-test' returned an invalid resource manifest"
    }

    It 'lists remote adapted metadata with <Payload> and reads its <SchemaShape> schema without trusting local paths' -ForEach @(
        @{ Payload = 'content'; SchemaShape = 'embedded' }
        @{ Payload = 'existing-path'; SchemaShape = 'direct' }
        @{ Payload = 'missing-path'; SchemaShape = 'embedded' }
    ) {
        $env:DSC_RESTRICTED_PATH = @($testDirectory, $discoverPath, $secretPath, $PSHOME) -join [IO.Path]::PathSeparator
        $localPath = Join-Path $testDirectory 'untrusted-path.json'
        if ($Payload -eq 'existing-path') {
            '{"untrusted":true}' | Set-Content -LiteralPath $localPath
        }
        $schema = @{ type = 'object'; properties = @{ adapted = @{ type = 'boolean' } } }
        $settings.registry = 'oci-test'
        $settings.requests = @()
        $settings.manifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/adapted/manifest.json'
            type = 'Test/SecretDiscovered'
            version = '0.1.0'
            kind = 'resource'
            condition = '[false]'
            capabilities = @('get', 'set')
            requireAdapter = 'Test/Adapter'
            schema = if ($SchemaShape -eq 'embedded') { @{ embedded = $schema } } else { $schema }
        }
        if ($Payload -eq 'content') {
            $settings.manifest.content = @{ adapted = $true }
        } else {
            $settings.manifest.path = $localPath
        }
        Set-RpcSettings
        $resources = @(Get-SecretDiscoveredResource)
        $resources | Should -HaveCount 1
        $resources[0].registry | Should -BeExactly 'oci-test'
        $resources[0].requireAdapter | Should -BeExactly 'Test/Adapter'
        $resources[0].capabilities | Should -BeExactly @('get', 'set')
        $resources[0].path | Should -BeNullOrEmpty
        $resources[0].directory | Should -BeNullOrEmpty
        Get-Content -Raw $TestDrive/error.log | Should -Not -Match 'invalid resource manifest'
        $actualSchema = dsc resource schema -r Test/SecretDiscovered 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $actualSchema.properties.adapted.type | Should -BeExactly 'boolean'
        dsc resource get -r Test/SecretDiscovered 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive/error.log | Should -Match 'not installed'
    }

    It 'preserves valid results around an invalid <Failure> discovery record' -ForEach @(
        @{ Failure = 'record' }
        @{ Failure = 'manifest' }
        @{ Failure = 'remote-path' }
    ) {
        $valid = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/Before'
            version = '0.1.0'
            get = @{ executable = 'not-installed' }
            schema = @{ embedded = @{ type = 'object' } }
        }
        $bad = switch ($Failure) {
            record { @{ registry = 'oci-test'; invalid = 'record' } }
            manifest { @{ registry = 'oci-test'; manifestContent = @{ type = 'invalid' } } }
            remote-path { @{ registry = 'oci-test'; manifestPath = (Join-Path $testDirectory 'remote.dsc.resource.json') } }
        }
        $after = $valid.Clone()
        $after.type = 'Test/After'
        $settings.requests = @()
        $settings.results = @(
            @{ registry = 'oci-test'; manifestContent = $valid }
            $bad
            @{ registry = 'oci-test'; manifestContent = $after }
        )
        Set-RpcSettings
        $resources = @(dsc resource list 'Test/*' 2>$TestDrive/error.log | ConvertFrom-Json)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $TestDrive/error.log)
        $resources.type | Should -Contain 'Test/Before'
        $resources.type | Should -Contain 'Test/After'
        Get-Content -Raw $TestDrive/error.log | Should -Match 'WARN'
    }

    It 'imports ordinary adapter metadata without evaluating its remote condition' {
        $settings.registry = 'oci-test'
        $settings.requests = @()
        $settings.manifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/SecretDiscovered'
            version = '0.1.0'
            kind = 'adapter'
            condition = '[false]'
            get = @{ executable = 'pwsh' }
            adapter = @{ inputKind = 'sequence'; list = @{ executable = 'pwsh' } }
            schema = @{ embedded = @{ type = 'object' } }
        }
        Set-RpcSettings
        $resource = Get-SecretDiscoveredResource
        $resource.kind | Should -BeExactly 'adapter'
        $resource.registry | Should -BeExactly 'oci-test'
        $resource.path | Should -BeNullOrEmpty
    }

    It 'retains valid registry metadata after rejecting another returned manifest' {
        $settings.registry = 'oci-test'
        $settings.requests = @()
        $settings.manifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/action/manifest.json'
            type = 'Test/SecretDiscovered'
            version = '0.1.0'
            invoke = @{ executable = 'pwsh' }
        }
        $settings.additionalManifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/SecretDiscovered'
            version = '0.1.0'
            get = @{ executable = 'pwsh' }
            schema = @{ embedded = @{ type = 'object' } }
        }
        Set-RpcSettings
        $resource = @(Get-SecretDiscoveredResource)
        $resource | Should -HaveCount 1
        $resource[0].registry | Should -BeExactly 'oci-test'
        Get-Content -LiteralPath $TestDrive\error.log -Raw | Should -Match "Registry 'oci-test' returned an invalid resource manifest"
    }

    It 'shows registry <ExpectedRegistry> after RequireAdapter in the resource table' -ForEach @(
        @{ Registry = $null; ExpectedRegistry = 'local' }
        @{ Registry = 'oci-test'; ExpectedRegistry = 'oci-test' }
    ) {
        $settings.registry = $Registry
        $settings.requests = @()
        Set-RpcSettings
        $output = @(dsc resource list 'Test/SecretDiscovered' --output-format table-no-truncate 2>$TestDrive\error.log)
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive\error.log -Raw)
        $header = @($output | Where-Object { $_ -match '^Type\s+Kind\s+' })[0]
        $header | Should -Match 'RequireAdapter\s+Registry\s+Description'
        $row = @($output | Where-Object { $_ -match '^Test/SecretDiscovered\s+' })[0]
        $row.Substring($header.IndexOf('Registry'), $ExpectedRegistry.Length) |
            Should -BeExactly $ExpectedRegistry
    }

    It 'does not execute an uninstalled registry resource manifest' {
        $settings.registry = 'oci-test'
        Set-RpcSettings
        dsc resource get -r Test/SecretDiscovered 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw $TestDrive/error.log | Should -Match 'not installed'
    }

    It 'reads an embedded schema without executing an uninstalled registry resource' {
        $settings.registry = 'oci-test'
        Set-RpcSettings
        $schema = dsc resource schema -r Test/SecretDiscovered 2>$TestDrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $schema.properties.authenticated.type | Should -BeExactly 'boolean'
    }

    It 'uses the JSON-RPC provider through the existing secret() function' {
        $env:PATH = @($secretPath, $previousPath) -join [IO.Path]::PathSeparator
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            directives = @{ reuseResources = $false }
            resources = @(
                @{
                    name = 'echo'; type = 'Microsoft.DSC.Debug/Echo'
                    properties = @{ showSecrets = $true; output = "[secret('$secretName')]" }
                }
            )
        }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 10) |
            ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0
        $result.results[0].result.actualState.output | Should -BeExactly 'test-runtime-token'
        $initialize = @((Get-ExtensionRpcEvent) | Where-Object {
            $_.role -eq 'secret' -and $_.event -eq 'initialize'
        })
        $initialize | Should -HaveCount 1
        $initialize[0].details.dscVersion | Should -BeExactly $result.executionInformation.version
        $initialize[0].details.reuseForbidden | Should -BeExactly '1'
    }

    It 'does not require Secret extensions when no secrets are configured' {
        $env:PATH = @($discoverPath, $previousPath) -join [IO.Path]::PathSeparator
        $settings.requests = @()
        Set-RpcSettings
        $resource = Get-SecretDiscoveredResource
        $resource.type | Should -BeExactly 'Test/SecretDiscovered'
        $resource.registry | Should -BeExactly 'local'
        @((Get-ExtensionRpcEvent) | Where-Object role -EQ secret) | Should -HaveCount 0
    }

    It 'reports missing secrets, wrong vaults, and absent providers for <Failure>' -ForEach @(
        @{ Failure = 'missing' }
        @{ Failure = 'vault' }
        @{ Failure = 'noProviders' }
    ) {
        $settings.requests = @(@{ name = $secretName; expected = $stringSecret })
        switch ($Failure) {
            missing { $settings.requests[0].name = 'missing-secret' }
            vault { $settings.requests[0].vault = 'missing-vault' }
            noProviders { $env:PATH = @($discoverPath, $previousPath) -join [IO.Path]::PathSeparator }
        }
        Set-RpcSettings
        Get-SecretDiscoveredResource | Should -BeNullOrEmpty
        Get-Content -LiteralPath $TestDrive/error.log -Raw |
            Should -Match 'Secret callback failed \(-32001\)'
    }

    It 'responds to <Case> without losing the pending discovery request' -ForEach @(
        @{ Case = 'unknownMethod'; Reference = @{ method = 'unsupported'; name = 'ignored'; expectedError = -32601 } }
        @{ Case = 'emptyName'; Reference = @{ name = ''; expectedError = -32602 } }
        @{ Case = 'invalidParams'; Reference = @{ params = @{ name = 42 }; expectedError = -32602 } }
        @{ Case = 'unknownParams'; Reference = @{ params = @{ name = 'ignored'; extra = $true }; expectedError = -32602 } }
    ) {
        $settings.requests = @($Reference, @{ name = $secretName; expected = $stringSecret })
        Set-RpcSettings
        (Get-SecretDiscoveredResource).type | Should -BeExactly 'Test/SecretDiscovered'
        @((Get-ExtensionRpcEvent) | Where-Object event -EQ callback) | Should -HaveCount 2
        @((Get-ExtensionRpcEvent) | Where-Object { $_.role -eq 'secret' -and $_.event -eq 'dsc_invoke' }) |
            Should -HaveCount 1
    }

    It 'rejects conflicting secret values from different providers' {
        $second = Get-Content -LiteralPath "$secretPath/secret.dsc.extension.json" -Raw |
            ConvertFrom-Json -AsHashtable
        $second.type = 'Test/JsonRpcSecretConflict'
        $second.secret.jsonrpc.args[-1] = Join-Path $secretPath secret.ps1
        $second.secret.jsonrpc.args += @('-ValueOverride', 'conflicting-test-value')
        $second | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath "$testDirectory/conflict.dsc.extension.json"
        $env:PATH = @($testDirectory, $env:PATH) -join [IO.Path]::PathSeparator
        $settings.requests = @(@{ name = $secretName; expectedError = -32001 })
        Set-RpcSettings
        (Get-SecretDiscoveredResource).type | Should -BeExactly 'Test/SecretDiscovered'
        ((Get-ExtensionRpcEvent) | Where-Object event -EQ callback).details.code | Should -Be -32001
    }

    It 'retrieves a secret from an existing command-based provider' {
        $legacyPath = (Resolve-Path "$PSScriptRoot/../../../extensions/test/secret").Path
        $env:PATH = @($legacyPath, $env:PATH) -join [IO.Path]::PathSeparator
        $settings.requests = @(@{ name = 'MySecret'; expected = @{ secureString = 'Hello' } })
        Set-RpcSettings
        (Get-SecretDiscoveredResource).type | Should -BeExactly 'Test/SecretDiscovered'
        @((Get-ExtensionRpcEvent) | Where-Object { $_.role -eq 'discover' -and $_.event -eq 'start' }) |
            Should -HaveCount 1
    }

    It 'returns an error for provider <Fault> without exposing invalid results' -ForEach @(
        @{ Fault = 'invalid' }
        @{ Fault = 'error' }
    ) {
        $env:DSC_TEST_SECRET_RPC_FAULT = $Fault
        $settings.requests = @(@{ name = $secretName; expectedError = -32001 })
        Set-RpcSettings
        (Get-SecretDiscoveredResource).type | Should -BeExactly 'Test/SecretDiscovered'
        Get-Content -LiteralPath $TestDrive/error.log -Raw |
            Should -Not -Match 'test-runtime-token|test-runtime-password'
    }

    It 'rejects recursive secret callbacks from a Secret provider' {
        $env:DSC_TEST_SECRET_RPC_FAULT = 'reverse'
        $settings.requests = @(@{ name = $secretName; expected = $stringSecret })
        Set-RpcSettings
        (Get-SecretDiscoveredResource).type | Should -BeExactly 'Test/SecretDiscovered'
        @((Get-ExtensionRpcEvent) | Where-Object event -EQ callbackRejected) | Should -HaveCount 1
    }

    It 'redacts <Transport> provider deprecation warnings only for callbacks' -ForEach @(
        @{ Transport = 'JSON-RPC' }
        @{ Transport = 'command' }
    ) {
        $provider = Get-Content -LiteralPath "$secretPath/secret.dsc.extension.json" -Raw |
            ConvertFrom-Json -AsHashtable
        $provider.type = 'Test/DeprecatedSecret'
        $provider.deprecationMessage = 'test-runtime-token test-runtime-password'
        if ($Transport -eq 'JSON-RPC') {
            $provider.secret.jsonrpc.args[-1] = Join-Path $secretPath secret.ps1
        } else {
            @'
param([string]$Name)
'test-runtime-token'
'@ | Set-Content -LiteralPath "$testDirectory/deprecated-secret.ps1"
            $provider.secret = @{
                executable = 'pwsh'
                args = @('-NoProfile', '-File', './deprecated-secret.ps1', @{ nameArg = '-Name' })
            }
        }
        $provider | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath "$testDirectory/deprecated.dsc.extension.json"
        $env:PATH = @($testDirectory, $discoverPath, $previousPath) -join [IO.Path]::PathSeparator
        $settings.requests = @(@{ name = $secretName; expected = $stringSecret })
        Set-RpcSettings
        (Get-SecretDiscoveredResource).type | Should -BeExactly 'Test/SecretDiscovered'
        Get-Content -LiteralPath $TestDrive/error.log -Raw |
            Should -Not -Match 'test-runtime-token|test-runtime-password'

        $env:PATH = @($testDirectory, $previousPath) -join [IO.Path]::PathSeparator
        $configuration = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name = 'echo'; type = 'Microsoft.DSC.Debug/Echo'
                    properties = @{ output = "[secret('$secretName')]" }
                }
            )
        }
        $null = dsc -l warn config get -i ($configuration | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath $TestDrive/error.log -Raw)
        Get-Content -LiteralPath $TestDrive/error.log -Raw |
            Should -Match ([regex]::Escape($provider.deprecationMessage))
    }

    It 'terminates malformed reverse requests and cleans up discovery' {
        $settings.requests = @(@{ name = $secretName; expected = $stringSecret })
        $settings.malformedId = $true
        Set-RpcSettings
        Get-SecretDiscoveredResource | Should -BeNullOrEmpty
        Get-Content -LiteralPath $TestDrive/error.log -Raw | Should -Match 'Invalid JSON-RPC'
    }

    It 'bounds a stalled <Transport> secret provider by the discovery request deadline' -ForEach @(
        @{ Transport = 'JSON-RPC' }
        @{ Transport = 'command' }
    ) {
        $manifest = Get-Content -LiteralPath "$discoverPath/discover.dsc.extension.json" -Raw |
            ConvertFrom-Json -AsHashtable
        $manifest.discover.jsonrpc.args[-1] = Join-Path $discoverPath discover.ps1
        $manifest.discover.jsonrpc.requestTimeoutMS = 2000
        $manifest | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath "$testDirectory/timeout.dsc.extension.json"
        $providerPath = $secretPath
        if ($Transport -eq 'command') {
            $providerPath = $testDirectory
            $protocolPath = (Join-Path $fixtureRoot protocol.ps1).Replace("'", "''")
            @"
param([string]`$Name)
. '$protocolPath'
Add-RpcEvent -Role secret -Event dsc_invoke
Start-Sleep -Seconds 30
"@ | Set-Content -LiteralPath "$testDirectory/slow-secret.ps1"
            $provider = Get-Content -LiteralPath "$secretPath/secret.dsc.extension.json" -Raw |
                ConvertFrom-Json -AsHashtable
            $provider.type = 'Test/SlowCommandSecret'
            $provider.secret = @{
                executable = 'pwsh'
                args = @('-NoProfile', '-File', './slow-secret.ps1', @{ nameArg = '-Name' })
            }
            $provider | ConvertTo-Json -Depth 20 |
                Set-Content -LiteralPath "$testDirectory/slow-secret.dsc.extension.json"
        }
        $env:PATH = @($testDirectory, $providerPath, $previousPath) -join [IO.Path]::PathSeparator
        $env:DSC_TEST_SECRET_RPC_FAULT = 'timeout'
        $stopwatch = [Diagnostics.Stopwatch]::StartNew()
        Get-SecretDiscoveredResource | Should -BeNullOrEmpty
        $stopwatch.Elapsed.TotalSeconds | Should -BeLessThan 10
        Get-Content -LiteralPath $TestDrive/error.log -Raw | Should -Match 'timed out'
        @((Get-ExtensionRpcEvent) | Where-Object { $_.role -eq 'secret' -and $_.event -eq 'dsc_invoke' }) |
            Should -HaveCount 1
    }

    It 'supports reverse requests from the resource JSON-RPC runner during <Invocation>' -ForEach @(
        @{ Invocation = 'standalone invocation' }
        @{ Invocation = 'configuration invocation' }
        @{ Invocation = 'configuration invocation with provider progress' }
    ) {
        @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/JsonRpcSecretConsumer'
            version = '0.1.0'
            jsonrpc = @{
                executable = 'pwsh'
                args = @('-NoProfile', '-File', (Join-Path $discoverPath discover.ps1), '-ResourceMode')
                initializeTimeoutMS = 5000
                shutdownTimeoutMS = 5000
            }
            get = @{}
            schema = @{ embedded = @{ type = 'object' } }
        } | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath "$testDirectory/consumer.dsc.resource.json"
        $env:PATH = @($testDirectory, $secretPath, $previousPath) -join [IO.Path]::PathSeparator
        $settings.numericIds = $true
        Set-RpcSettings
        if ($Invocation -eq 'standalone invocation') {
            $result = dsc resource get -r Test/JsonRpcSecretConsumer | ConvertFrom-Json -Depth 30
            $LASTEXITCODE | Should -Be 0
            $result.actualState.authenticated | Should -BeTrue
        } else {
            if ($Invocation -eq 'configuration invocation with provider progress') {
                $env:DSC_TEST_SECRET_RPC_FAULT = 'progress'
            }
            $configuration = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
                directives = @{ reuseResources = $false }
                resources = @(@{ name = 'consumer'; type = 'Test/JsonRpcSecretConsumer' })
            }
            $result = dsc --progress-format json config get -i ($configuration | ConvertTo-Json -Depth 10) 2>$TestDrive/error.log |
                ConvertFrom-Json -Depth 30
            $LASTEXITCODE | Should -Be 0
            $result.hadErrors | Should -BeFalse
            $result.results[0].result.actualState.authenticated | Should -BeTrue
            $diagnostics = Get-Content -LiteralPath $TestDrive/error.log -Raw
            $diagnostics | Should -Not -Match 'test-runtime-token|test-runtime-password'
            if ($Invocation -eq 'configuration invocation with provider progress') {
                $progress = @(Get-Content -LiteralPath $TestDrive/error.log | ConvertFrom-Json |
                    Where-Object resourceProgress)
                $progress | Should -HaveCount 2
                foreach ($record in $progress) {
                    $record.resourceProgress.completed | Should -Be 1
                    $record.resourceProgress.total | Should -Be 1
                    $record.activity | Should -Not -BeNullOrEmpty
                }
            }
            $initialize = @((Get-ExtensionRpcEvent) | Where-Object {
                $_.role -eq 'secret' -and $_.event -eq 'initialize'
            })
            $initialize | Should -HaveCount 2
            foreach ($event in $initialize) {
                $event.details.dscVersion | Should -BeExactly $result.executionInformation.version
                $event.details.reuseForbidden | Should -BeExactly '1'
            }
        }
        @((Get-ExtensionRpcEvent) | Where-Object event -EQ callback) | Should -HaveCount 2
    }

    It 'validates registry discovery results with <SchemaPath>' -ForEach @(
        @{ SchemaPath = 'extension\stdout\discover.json' }
        @{ SchemaPath = 'bundled\extension\stdout\discover.json' }
        @{ SchemaPath = 'bundled\extension\stdout\discover.vscode.json' }
    ) {
        $schemaPath = Join-Path "$PSScriptRoot\..\..\..\schemas\v3" $SchemaPath
        $schema = Get-Content -LiteralPath $schemaPath -Raw
        foreach ($value in @(
            @{ manifestPath = (Join-Path $testDirectory 'resource.dsc.resource.json'); registry = 'oci-test' }
            @{ manifestContent = @{ type = 'Test/Example'; version = '1.0.0' }; registry = 'oci-test' }
        )) {
            $value | ConvertTo-Json -Depth 10 | Test-Json -Schema $schema | Should -BeTrue
        }
        foreach ($value in @(
            @{ registry = 'oci-test' }
            @{ manifestPath = 'example'; manifestContent = @{}; registry = 'oci-test' }
            @{ manifestContent = @{}; registry = 42 }
        )) {
            $value | ConvertTo-Json -Depth 10 |
                Test-Json -Schema $schema -ErrorAction SilentlyContinue | Should -BeFalse
        }
    }

    It 'validates JSON-RPC extension manifests with the <SchemaKind> schema' -ForEach @(
        @{ SchemaKind = 'engine' }
        @{ SchemaKind = 'published bundle' }
        @{ SchemaKind = 'authoring bundle' }
        @{ SchemaKind = 'published extension-list bundle' }
        @{ SchemaKind = 'authoring extension-list bundle' }
    ) {
        $schema = switch ($SchemaKind) {
            engine {
                dsc schema -t extension-manifest | Out-String
                $LASTEXITCODE | Should -Be 0
            }
            'published bundle' {
                Get-Content -LiteralPath "$PSScriptRoot/../../../schemas/v3/bundled/extension/manifest.json" -Raw
            }
            'authoring bundle' {
                Get-Content -LiteralPath "$PSScriptRoot/../../../schemas/v3/bundled/extension/manifest.vscode.json" -Raw
            }
            'published extension-list bundle' {
                Get-Content -LiteralPath "$PSScriptRoot/../../../schemas/v3/bundled/outputs/extension/list.json" -Raw
            }
            'authoring extension-list bundle' {
                Get-Content -LiteralPath "$PSScriptRoot/../../../schemas/v3/bundled/outputs/extension/list.vscode.json" -Raw
            }
        }
        foreach ($path in @("$discoverPath/discover.dsc.extension.json", "$secretPath/secret.dsc.extension.json")) {
            $manifest = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json -AsHashtable
            $value = if ($SchemaKind -like '*extension-list*') { @{ manifest = $manifest } } else { $manifest }
            $value | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema | Should -BeTrue
        }
        foreach ($discover in @(
            @{ executable = 'pwsh'; network = $true }
            @{ jsonrpc = @{ executable = 'pwsh' }; network = $true }
        )) {
            $manifest = Get-Content (Join-Path $discoverPath discover.dsc.extension.json) -Raw |
                ConvertFrom-Json -AsHashtable
            $manifest.discover = $discover
            $value = if ($SchemaKind -like '*extension-list*') { @{ manifest = $manifest } } else { $manifest }
            $value | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema | Should -BeTrue
            $discover.network = 'true'
            $value | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema -ErrorAction SilentlyContinue |
                Should -BeFalse
        }
        $manifest = Get-Content -LiteralPath "$secretPath/secret.dsc.extension.json" -Raw |
            ConvertFrom-Json -AsHashtable
        $manifest.secret.executable = 'pwsh'
        $value = if ($SchemaKind -like '*extension-list*') { @{ manifest = $manifest } } else { $manifest }
        $value | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema -ErrorAction SilentlyContinue |
            Should -BeFalse
    }
}
