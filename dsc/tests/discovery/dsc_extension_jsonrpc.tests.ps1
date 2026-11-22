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
    }

    BeforeEach {
        $previousPath = $env:PATH
        $previousResourcePath = $env:DSC_RESOURCE_PATH
        $previousLog = $env:DSC_TEST_EXTENSION_RPC_LOG
        $previousSettings = $env:DSC_TEST_EXTENSION_RPC_SETTINGS
        $previousFault = $env:DSC_TEST_SECRET_RPC_FAULT
        $env:PATH = @($discoverPath, $secretPath, $previousPath) -join [IO.Path]::PathSeparator
        $env:DSC_RESOURCE_PATH = $null
        $env:DSC_TEST_EXTENSION_RPC_LOG = Join-Path $TestDrive "$([guid]::NewGuid()).jsonl"
        $env:DSC_TEST_EXTENSION_RPC_SETTINGS = Join-Path $TestDrive "$([guid]::NewGuid()).json"
        $env:DSC_TEST_SECRET_RPC_FAULT = $null
        $testDirectory = (New-Item -ItemType Directory -Path "$TestDrive/$([guid]::NewGuid())").FullName
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
        $env:DSC_RESOURCE_PATH = $previousResourcePath
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

    It 'preserves the registry friendly name from a Discover result' {
        $settings.registry = 'oci-test'
        Set-RpcSettings
        (Get-SecretDiscoveredResource).registry | Should -BeExactly 'oci-test'
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
        $manifest = Get-Content -LiteralPath "$secretPath/secret.dsc.extension.json" -Raw |
            ConvertFrom-Json -AsHashtable
        $manifest.secret.executable = 'pwsh'
        $value = if ($SchemaKind -like '*extension-list*') { @{ manifest = $manifest } } else { $manifest }
        $value | ConvertTo-Json -Depth 30 | Test-Json -Schema $schema -ErrorAction SilentlyContinue |
            Should -BeFalse
    }
}
