# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'JSON-RPC resource lifecycle' {
    BeforeAll {
        function New-RpcConfiguration {
            param(
                [string]$ResourceType = 'Test/JsonRpc',
                [hashtable]$Properties = @{},
                [hashtable]$Directives = @{}
            )
            @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
                directives = $Directives
                resources = @(
                    @{ name = 'first'; type = $ResourceType; properties = $Properties }
                    @{ name = 'second'; type = $ResourceType; properties = $Properties }
                )
            }
        }

        function Get-RpcEvent {
            if (Test-Path -LiteralPath $env:DSC_TEST_JSONRPC_LOG) {
                Get-Content -LiteralPath $env:DSC_TEST_JSONRPC_LOG | ConvertFrom-Json
            }
        }
    }

    BeforeEach {
        $previousLog = $env:DSC_TEST_JSONRPC_LOG
        $previousReuse = $env:DSC_RESOURCE_REUSE
        $previousFault = $env:DSC_TEST_JSONRPC_FAULT
        $env:DSC_TEST_JSONRPC_LOG = Join-Path $TestDrive "$([guid]::NewGuid()).jsonl"
        $env:DSC_RESOURCE_REUSE = $null
        $env:DSC_TEST_JSONRPC_FAULT = $null
    }

    AfterEach {
        foreach ($processId in (Get-RpcEvent).pid | Select-Object -Unique) {
            Get-Process -Id $processId -ErrorAction SilentlyContinue | Should -BeNullOrEmpty
        }
        $env:DSC_TEST_JSONRPC_LOG = $previousLog
        $env:DSC_RESOURCE_REUSE = $previousReuse
        $env:DSC_TEST_JSONRPC_FAULT = $previousFault
    }

    It 'reuses one process for schema, validation, and all instances until completion' {
        $configuration = New-RpcConfiguration
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.hadErrors | Should -BeFalse
        $states = $result.results.result.actualState
        $states | Should -HaveCount 2
        $states[0].processId | Should -Be $states[1].processId
        $states[1].requestCount | Should -BeGreaterThan $states[0].requestCount
        $events = @(Get-RpcEvent)
        @($events | Where-Object event -EQ initialize) | Should -HaveCount 1
        @($events | Where-Object event -EQ shutdown) | Should -HaveCount 1
        $events[-1].event | Should -BeExactly 'exit'
        $operations = ($events | Where-Object event -EQ dsc_invoke).params.operation
        $operations | Should -Contain 'schema'
        $operations | Should -Contain 'get'
    }

    It 'shares the process with a custom validation operation' {
        $configuration = New-RpcConfiguration -ResourceType 'Test/JsonRpcValidate'
        $null = dsc config get -i ($configuration | ConvertTo-Json -Depth 20)
        $LASTEXITCODE | Should -Be 0
        $events = @(Get-RpcEvent)
        @($events | Where-Object event -EQ initialize) | Should -HaveCount 1
        ($events | Where-Object event -EQ dsc_invoke).params.operation | Should -Contain 'validate'
    }

    It 'validates JSON-RPC manifests against the published bundle' {
        $schemaPath = Join-Path $PSScriptRoot '../../../schemas/v3/bundled/resource/manifest.json'
        $manifestPath = Join-Path $PSScriptRoot '../../../tools/dsctest/dsctest.dsc.manifests.json'
        $manifests = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json -AsHashtable
        foreach ($manifest in $manifests.resources | Where-Object jsonrpc) {
            $manifest | ConvertTo-Json -Depth 30 | Test-Json -SchemaFile $schemaPath | Should -BeTrue
        }
        $invalid = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/MissingTransport'
            version = '0.1.0'
            get = @{}
        }
        $invalid | ConvertTo-Json -Depth 10 | Test-Json -SchemaFile $schemaPath -ErrorAction SilentlyContinue |
            Should -BeFalse
    }

    It 'continues without metadata rejected by custom validation' {
        $configuration = New-RpcConfiguration -ResourceType 'Test/JsonRpcValidate' -Properties @{ rejectMetadata = $true }
        $configuration.resources[0].metadata = @{ example = 'metadata probe' }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0
        $result.hadErrors | Should -BeFalse
        $calls = @((Get-RpcEvent) | Where-Object { $_.event -eq 'dsc_invoke' -and $_.params.operation -eq 'get' })
        $calls | Should -HaveCount 2
        $calls[0].params.properties._metadata | Should -BeNullOrEmpty
        @((Get-RpcEvent) | Where-Object event -EQ initialize) | Should -HaveCount 1
    }

    It 'does not reuse a process across configuration runs' {
        $configuration = New-RpcConfiguration | ConvertTo-Json -Depth 20
        $first = dsc config get -i $configuration | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $second = dsc config get -i $configuration | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $first.results[0].result.actualState.processId | Should -Not -Be $second.results[0].result.actualState.processId
        @((Get-RpcEvent) | Where-Object event -EQ initialize) | Should -HaveCount 2
    }

    It 'applies reuse restrictions during config validate from <Scope>' -ForEach @(
        @{ Scope = 'document' }
        @{ Scope = 'instance' }
    ) {
        $configuration = New-RpcConfiguration -ResourceType 'Test/JsonRpcValidate'
        if ($Scope -eq 'document') {
            $configuration.directives.reuseResources = $false
        } else {
            foreach ($resource in $configuration.resources) {
                $resource.directives = @{ reuseResource = $false }
            }
        }
        $null = dsc config validate -i ($configuration | ConvertTo-Json -Depth 20)
        $LASTEXITCODE | Should -Be 0
        @((Get-RpcEvent) | Where-Object event -EQ initialize) | Should -HaveCount 0
        @((Get-RpcEvent) | Where-Object event -EQ fallback) | Should -HaveCount 2
    }

    It 'preserves JSON-RPC <Fault> errors while probing metadata for <ResourceType>' -ForEach @(
        @{ ResourceType = 'Test/JsonRpc'; Fault = 'initializeError' }
        @{ ResourceType = 'Test/JsonRpcValidate'; Fault = 'initializeError' }
        @{ ResourceType = 'Test/JsonRpcValidate'; Fault = 'error' }
    ) {
        $configuration = New-RpcConfiguration -ResourceType $ResourceType -Properties @{ fault = $Fault }
        $configuration.resources[0].metadata = @{ example = 'metadata probe' }
        if ($Fault -eq 'initializeError') {
            $env:DSC_TEST_JSONRPC_FAULT = $Fault
        }
        $output = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) 2>&1 | Out-String
        $LASTEXITCODE | Should -Not -Be 0
        $output | Should -Match 'Requested (lifecycle|test) error'
        $output | Should -Not -Match 'scope has already failed'
        @((Get-RpcEvent) | Where-Object { $_.event -eq 'dsc_invoke' -and $_.params.operation -eq 'get' }) |
            Should -HaveCount 0
    }

    It 'includes JSON-RPC application failures in JSON progress' {
        $configuration = New-RpcConfiguration -Properties @{ fault = 'error'; errorCode = -32001 }
        $progressPath = Join-Path $TestDrive 'failure-progress.jsonl'
        $null = dsc --progress-format json config get -i ($configuration | ConvertTo-Json -Depth 20) 2> $progressPath
        $LASTEXITCODE | Should -Not -Be 0
        $records = @(Get-Content $progressPath | Where-Object { $_.StartsWith('{') } | ConvertFrom-Json)
        $failures = @($records | Where-Object failure)
        $failures.Count | Should -BeGreaterThan 0
        $failures[0].failure.exitCode | Should -Be -32001
        $failures[0].failure.message | Should -BeExactly 'Requested test error'
    }

    It 'restarts persistent hosts after a Windows environment refresh' -Skip:(!$IsWindows) {
        $variableName = "DSC_JSONRPC_$([guid]::NewGuid().ToString('N'))"
        $configuration = New-RpcConfiguration -Properties @{ environmentName = $variableName; value = 5 }
        $configuration.resources = @(
            $configuration.resources[0]
            @{ name = 'refresh'; type = 'Test/RefreshEnv'; properties = @{ name = $variableName; value = 'refreshed' } }
            $configuration.resources[1]
        )
        try {
            $result = dsc config set -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json -Depth 30
            $LASTEXITCODE | Should -Be 0
            $before = $result.results[0].result.afterState
            $after = $result.results[2].result.afterState
            $before.environmentValue | Should -BeNullOrEmpty
            $after.environmentValue | Should -BeExactly 'refreshed'
            $before.processId | Should -Not -Be $after.processId
            @((Get-RpcEvent) | Where-Object event -EQ initialize) | Should -HaveCount 2
            @((Get-RpcEvent) | Where-Object event -EQ shutdown) | Should -HaveCount 2
        } finally {
            [System.Environment]::SetEnvironmentVariable($variableName, $null, [System.EnvironmentVariableTarget]::User)
        }
    }

    It 'shares one adapter host across target resources during <Operation>' -ForEach @(
        @{ Operation = 'get' }
        @{ Operation = 'set' }
        @{ Operation = 'test' }
        @{ Operation = 'export' }
    ) {
        $configuration = New-RpcConfiguration -ResourceType 'Adapted/JsonRpcOne' -Properties @{ value = 5 }
        $configuration.resources[1].type = 'Adapted/JsonRpcTwo'
        $result = dsc config $Operation -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0
        $events = @(Get-RpcEvent)
        @($events | Where-Object event -EQ initialize) | Should -HaveCount 1
        $invocations = @($events | Where-Object event -EQ dsc_invoke)
        $targets = @($invocations | Where-Object { $_.params.operation -eq $Operation })
        $targets | Should -HaveCount 2
        $targets[0].params.resourceType | Should -BeExactly 'Adapted/JsonRpcOne'
        $targets[1].params.resourceType | Should -BeExactly 'Adapted/JsonRpcTwo'
        $targets[0].params.context.targetResource.type | Should -BeExactly 'Adapted/JsonRpcOne'
        $targets[1].params.context.targetResource.path | Should -BeExactly 'jsonrpc/Two'
        $targets[0].params.context.targetResource.version | Should -BeExactly '0.1.0'
        $targets[0].params.properties.value | Should -Be 5
        if ($Operation -eq 'export') {
            @($invocations | Where-Object { $_.params.context.schemaKind -eq 'exportInput' }) | Should -HaveCount 2
            $result.resources | Should -HaveCount 2
        } else {
            $result.hadErrors | Should -BeFalse
        }
        $events[-1].event | Should -BeExactly 'exit'
    }

    It 'uses isolated RPC calls when document reuse is disabled' {
        $configuration = New-RpcConfiguration -Directives @{ reuseResources = $false }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.results[0].result.actualState.processId | Should -Not -Be $result.results[1].result.actualState.processId
        $invocations = @((Get-RpcEvent) | Where-Object event -EQ dsc_invoke)
        @($invocations.pid | Select-Object -Unique) | Should -HaveCount $invocations.Count
        @((Get-RpcEvent) | Where-Object event -EQ shutdown) | Should -HaveCount $invocations.Count
    }

    It 'prefers RPC over fallback unless reuse is disabled by <Scope>' -ForEach @(
        @{ Scope = 'setting' }
        @{ Scope = 'document' }
        @{ Scope = 'instance' }
    ) {
        $configuration = New-RpcConfiguration -ResourceType 'Test/JsonRpcFallback'
        $preferred = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $preferred.results[0].result.actualState.transport | Should -BeExactly 'jsonrpc'
        switch ($Scope) {
            'setting' { $env:DSC_RESOURCE_REUSE = 'false' }
            'document' { $configuration.directives.reuseResources = $false }
            'instance' {
                foreach ($resource in $configuration.resources) {
                    $resource.directives = @{ reuseResource = $false }
                }
            }
        }
        $isolated = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $isolated.results[0].result.actualState.transport | Should -BeExactly 'executable'
        $isolated.results[1].result.actualState.transport | Should -BeExactly 'executable'
    }

    It 'does not let instance reuse override a document prohibition' {
        $configuration = New-RpcConfiguration -ResourceType 'Test/JsonRpcFallback' -Directives @{ reuseResources = $false }
        $configuration.resources[0].directives = @{ reuseResource = $true }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.results[0].result.actualState.transport | Should -BeExactly 'executable'
        @((Get-RpcEvent) | Where-Object event -EQ initialize) | Should -HaveCount 0
    }

    It 'propagates a document prohibition but not an instance prohibition into nested configurations' -ForEach @(
        @{ DocumentReuse = $false; ExpectedTransport = 'executable' }
        @{ DocumentReuse = $true; ExpectedTransport = 'jsonrpc' }
    ) {
        $nested = New-RpcConfiguration -ResourceType 'Test/JsonRpcFallback'
        $configuration = New-RpcConfiguration -Directives @{ reuseResources = $DocumentReuse }
        $configuration.resources = @(@{
            name = 'nested'
            type = 'Microsoft.DSC/Group'
            directives = @{ reuseResource = $false }
            properties = $nested
        })
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 30) | ConvertFrom-Json -Depth 30
        $LASTEXITCODE | Should -Be 0
        $result.hadErrors | Should -BeFalse
        $states = $result.results[0].result.result.actualState
        $states | Should -HaveCount 2
        $states[0].transport | Should -BeExactly $ExpectedTransport
        $states[1].transport | Should -BeExactly $ExpectedTransport
        if ($DocumentReuse) {
            $states[0].processId | Should -Be $states[1].processId
        }
    }

    It 'initializes with the effective tracing, DSC version, security context, and configuration root' {
        $configuration = New-RpcConfiguration
        $configurationPath = Join-Path $TestDrive 'initialize.dsc.json'
        $configuration | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $configurationPath
        $result = dsc --trace-level debug --trace-format json config get -f $configurationPath 2> (Join-Path $TestDrive 'initialize-trace.jsonl') |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $initialize = @((Get-RpcEvent) | Where-Object event -EQ initialize)
        $initialize | Should -HaveCount 1
        $initialize[0].params.protocolVersions | Should -Be @(1)
        $initialize[0].params.traceLevel | Should -BeExactly 'debug'
        $initialize[0].params.traceFormat | Should -BeExactly 'json'
        $initialize[0].params.dscVersion | Should -BeExactly $result.executionInformation.version
        $initialize[0].params.securityContext | Should -BeIn @('elevated', 'restricted')
        (Resolve-Path $initialize[0].params.configRoot).Path | Should -BeExactly (Resolve-Path $TestDrive).Path
    }

    It 'keeps the pooled process when another instance disables reuse' {
        $configuration = New-RpcConfiguration
        $configuration.resources[1].directives = @{ reuseResource = $false }
        $configuration.resources += @{ name = 'third'; type = 'Test/JsonRpc'; properties = @{} }
        $result = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $states = $result.results.result.actualState
        $states[0].processId | Should -Not -Be $states[1].processId
        $states[0].processId | Should -Be $states[2].processId
        @((Get-RpcEvent) | Where-Object event -EQ initialize) | Should -HaveCount 2
    }

    It 'does not apply executable-only environment input conversion to RPC properties' {
        $result = dsc resource get -r Test/JsonRpcFallback -i '{"nested":{"value":1}}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.actualState.nested.value | Should -Be 1
        $result.actualState.transport | Should -BeExactly 'jsonrpc'
    }

    It 'preserves before state, native set, and stateAndDiff semantics' {
        $configuration = New-RpcConfiguration -Properties @{ value = 5 }
        $result = dsc config set -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $first = $result.results[0].result
        $second = $result.results[1].result
        $first.beforeState.value | Should -Be 0
        $first.afterState.value | Should -Be 5
        $first.changedProperties | Should -Contain 'value'
        $second.beforeState.value | Should -Be 5
        $second.afterState.processId | Should -Be $first.afterState.processId
    }

    It 'does not mutate resource state during native what-if' {
        $configuration = New-RpcConfiguration -Properties @{ value = 5 }
        $result = dsc config set -w -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.results[0].result.afterState.value | Should -Be 5
        $result.results[1].result.beforeState.value | Should -Be 0
        $mutations = @((Get-RpcEvent) | Where-Object { $_.event -eq 'dsc_invoke' -and $_.params.operation -eq 'set' })
        $mutations | Should -HaveCount 2
        $mutations.params.executionType | Select-Object -Unique | Should -BeExactly 'whatIf'
    }

    It 'normalizes native test and export results' {
        $test = dsc resource test -r Test/JsonRpc -i '{"value":5}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $test.inDesiredState | Should -BeFalse
        $test.differingProperties | Should -Contain 'value'
        $export = dsc resource export -r Test/JsonRpc | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $export.resources | Should -HaveCount 1
        $export.resources[0].properties.transport | Should -BeExactly 'jsonrpc'
    }

    It 'uses <ExpectedOperation> for hybrid <Operation> what-if with reuse <Reuse> and separate what-if <SeparateWhatIf>' -ForEach @(
        @{ Operation = 'set'; Reuse = $true; SeparateWhatIf = $false; ExpectedOperation = 'test' }
        @{ Operation = 'set'; Reuse = $true; SeparateWhatIf = $true; ExpectedOperation = 'whatIf' }
        @{ Operation = 'set'; Reuse = $false; SeparateWhatIf = $false; ExpectedOperation = 'fallback' }
        @{ Operation = 'delete'; Reuse = $true; SeparateWhatIf = $false; ExpectedOperation = 'test' }
        @{ Operation = 'delete'; Reuse = $false; SeparateWhatIf = $false; ExpectedOperation = 'fallback' }
    ) {
        $manifest = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type = 'Test/JsonRpcHybridWhatIf'
            version = '0.1.0'
            jsonrpc = @{ executable = 'dsctest'; args = @('jsonrpc'); initializeTimeoutMS = 2000 }
            get = @{}
            test = @{ return = 'stateAndDiff' }
            set = @{
                executable = 'dsctest'
                args = @('whatif', '--state-and-diff', @{ whatIfArg = '-w' })
                return = 'stateAndDiff'
            }
            delete = @{
                executable = 'dsctest'
                args = @('whatif-delete', @{ whatIfArg = '-w' })
            }
            schema = @{ embedded = @{ type = 'object'; additionalProperties = $true } }
        }
        if ($SeparateWhatIf) {
            $manifest.whatIf = @{ return = 'stateAndDiff' }
        }
        $manifestPath = Join-Path $TestDrive 'hybrid.dsc.resource.json'
        $manifest | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $manifestPath
        $properties = @{ value = 5 }
        if ($Operation -eq 'delete') {
            $properties._exist = $false
        }
        $configuration = New-RpcConfiguration -ResourceType $manifest.type -Properties $properties -Directives @{ reuseResources = $Reuse }
        $previousResourcePath = $env:DSC_RESOURCE_PATH
        try {
            $env:DSC_RESOURCE_PATH = (@($TestDrive, $previousResourcePath) | Where-Object { $_ }) -join [IO.Path]::PathSeparator
            $result = dsc config set -w -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json -Depth 30
            $LASTEXITCODE | Should -Be 0
            $result.hadErrors | Should -BeFalse
            $calls = @((Get-RpcEvent) | Where-Object event -EQ dsc_invoke)
            $calls.params.operation | Should -Not -Contain 'set'
            $calls.params.operation | Should -Not -Contain 'delete'
            if ($ExpectedOperation -eq 'fallback') {
                if ($Operation -eq 'set') {
                    $result.results[0].result.afterState.executionType | Should -BeExactly 'WhatIf'
                    $result.results[0].result.afterState.fromResource | Should -BeExactly 'ResourceProvidedDiff'
                } else {
                    $result.results[0].metadata.whatIf | Should -Contain 'Delete what-if message 1'
                }
            } else {
                $selectedCalls = @($calls | Where-Object { $_.params.operation -eq $ExpectedOperation })
                $selectedCalls | Should -HaveCount 2
                if ($SeparateWhatIf) {
                    $selectedCalls.params.executionType | Select-Object -Unique | Should -BeExactly 'whatIf'
                }
            }
        } finally {
            $env:DSC_RESOURCE_PATH = $previousResourcePath
        }
    }

    It 'preserves follow-up get, synthetic export, and synthetic what-if semantics' {
        $set = dsc resource set -r Test/JsonRpcSynthetic -i '{"value":5}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $set.beforeState.value | Should -Be 0
        $set.afterState.value | Should -Be 5
        $set.beforeState.processId | Should -Be $set.afterState.processId
        $test = dsc resource test -r Test/JsonRpcSynthetic -i '{"value":5}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $test.inDesiredState | Should -BeFalse
        $export = dsc resource export -r Test/JsonRpcSynthetic | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $export.resources | Should -HaveCount 1
        $configuration = New-RpcConfiguration -ResourceType 'Test/JsonRpcSynthetic' -Properties @{ value = 5 }
        $whatIf = dsc config set -w -i ($configuration | ConvertTo-Json -Depth 20) | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $whatIf.results[0].result.afterState.value | Should -Be 5
    }

    It 'does not send an actual delete when standalone set routes what-if through delete' {
        $null = dsc resource set -w -r Test/JsonRpc -i '{"_exist":false}'
        $LASTEXITCODE | Should -Be 0
        $deletes = @((Get-RpcEvent) | Where-Object { $_.event -eq 'dsc_invoke' -and $_.params.operation -eq 'delete' })
        $deletes | Should -HaveCount 1
        $deletes[0].params.executionType | Should -BeExactly 'whatIf'
    }

    It 'cleans up standalone resource <Operation> before returning' -ForEach @(
        @{ Operation = 'get' }
        @{ Operation = 'set' }
        @{ Operation = 'test' }
        @{ Operation = 'delete' }
        @{ Operation = 'schema' }
        @{ Operation = 'export' }
    ) {
        if ($Operation -in 'schema', 'export') {
            $null = dsc resource $Operation -r Test/JsonRpc
        } else {
            $null = dsc resource $Operation -r Test/JsonRpc -i '{"value":5}'
        }
        $LASTEXITCODE | Should -Be 0
        $events = @(Get-RpcEvent)
        @($events | Where-Object event -EQ initialize) | Should -HaveCount 1
        $events[-2].event | Should -BeExactly 'shutdown'
        $events[-1].event | Should -BeExactly 'exit'
    }

    It 'gracefully shuts down after error code <Code> without invoking another instance' -ForEach @(
        @{ Code = 0 }
        @{ Code = 7 }
        @{ Code = -32601 }
        @{ Code = -32001 }
    ) {
        $configuration = New-RpcConfiguration -Properties @{ fault = 'error'; errorCode = $Code }
        $output = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        $output | Out-String | Should -Match 'Requested test error'
        $events = @(Get-RpcEvent)
        @($events | Where-Object { $_.event -eq 'dsc_invoke' -and $_.params.properties.fault -eq 'error' }) | Should -HaveCount 1
        $events[-1].event | Should -BeExactly 'exit'
    }

    It 'fails and terminates the child for <Fault>' -ForEach @(
        @{ Fault = 'malformed' }
        @{ Fault = 'wrongId' }
        @{ Fault = 'exit' }
        @{ Fault = 'nullId' }
        @{ Fault = 'missingId' }
        @{ Fault = 'both' }
        @{ Fault = 'batch' }
        @{ Fault = 'duplicate' }
    ) {
        $null = dsc resource get -r Test/JsonRpc -i (@{ fault = $Fault } | ConvertTo-Json) 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        @((Get-RpcEvent) | Where-Object event -EQ shutdown) | Should -HaveCount 0
    }

    It 'rejects invalid operation result shapes without invoking another instance' {
        $configuration = New-RpcConfiguration -Properties @{ fault = 'invalidResult' }
        $null = dsc config get -i ($configuration | ConvertTo-Json -Depth 20) 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        @((Get-RpcEvent) | Where-Object { $_.event -eq 'dsc_invoke' -and $_.params.operation -eq 'get' }) | Should -HaveCount 1
        (Get-RpcEvent)[-1].event | Should -BeExactly 'exit'
    }

    It 'fails and cleans up for lifecycle fault <Fault>' -ForEach @(
        @{ Fault = 'initializeError' }
        @{ Fault = 'initializeVersion' }
        @{ Fault = 'initializeTimeout' }
        @{ Fault = 'shutdownError' }
        @{ Fault = 'shutdownTimeout' }
        @{ Fault = 'exitTimeout' }
        @{ Fault = 'exitOutput' }
    ) {
        $env:DSC_TEST_JSONRPC_FAULT = $Fault
        $timer = [System.Diagnostics.Stopwatch]::StartNew()
        $null = dsc resource get -r Test/JsonRpc 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        $timer.Elapsed.TotalSeconds | Should -BeLessThan 6
        if ($Fault.StartsWith('initialize')) {
            @((Get-RpcEvent) | Where-Object event -EQ dsc_invoke) | Should -HaveCount 0
        }
    }

    It 'preserves the operation failure when shutdown also fails' {
        $env:DSC_TEST_JSONRPC_FAULT = 'shutdownError'
        $output = dsc resource get -r Test/JsonRpc -i '{"fault":"error"}' 2>&1 | Out-String
        $LASTEXITCODE | Should -Not -Be 0
        $output | Should -Match 'Requested test error'
    }

    It 'rejects invalid progress <Case>' -ForEach @(
        @{ Case = 'negative completed'; Progress = @(@{ completed = -1 }) }
        @{ Case = 'zero total'; Progress = @(@{ completed = 0; total = 0 }) }
        @{ Case = 'total without completed'; Progress = @(@{ total = 1 }) }
        @{ Case = 'completed above total'; Progress = @(@{ completed = 2; total = 1 }) }
        @{ Case = 'regressing completed'; Progress = @(@{ completed = 1 }, @{ completed = 0 }) }
        @{ Case = 'changing total'; Progress = @(@{ completed = 0; total = 1 }, @{ completed = 0; total = 2 }) }
        @{ Case = 'completed above previous total'; Progress = @(@{ completed = 0; total = 1 }, @{ completed = 2 }) }
        @{ Case = 'wrong request'; Progress = @(@{ requestId = 0 }) }
    ) {
        $null = dsc resource get -r Test/JsonRpc -i (@{ progress = $Progress } | ConvertTo-Json -Depth 20) 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        @((Get-RpcEvent) | Where-Object event -EQ shutdown) | Should -HaveCount 0
    }

    It 'does not reset request deadlines when progress notifications arrive' {
        $output = dsc resource get -r Test/JsonRpc -i '{"notificationDelayMS":100}' 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        $output | Out-String | Should -Match 'timed out'
        @((Get-RpcEvent) | Where-Object event -EQ shutdown) | Should -HaveCount 0
    }

    It 'enforces the request deadline and terminates an unresponsive resource' {
        $output = dsc resource get -r Test/JsonRpc -i '{"delayMS":10000}' 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        $output | Out-String | Should -Match 'timed out'
        @((Get-RpcEvent) | Where-Object event -EQ shutdown) | Should -HaveCount 0
        @((Get-RpcEvent) | Where-Object event -EQ shutdown) | Should -HaveCount 0
    }

    It 'applies explicit requestTimeoutMS <Timeout> instead of the manifest default' -ForEach @(
        @{ Timeout = 150; Delay = 600 }
        @{ Timeout = 0; Delay = 1200 }
    ) {
        $settingsDirectory = New-Item -ItemType Directory -Path (Join-Path $TestDrive ([guid]::NewGuid()))
        @{ resourceTimeouts = @{ requestTimeoutMS = $Timeout } } | ConvertTo-Json |
            Set-Content -LiteralPath (Join-Path $settingsDirectory.FullName 'dsc.settings.json')
        Push-Location $settingsDirectory.FullName
        try {
            $output = dsc resource get -r Test/JsonRpc -i (@{ delayMS = $Delay } | ConvertTo-Json) 2>&1
            if ($Timeout -eq 0) {
                $LASTEXITCODE | Should -Be 0
                ($output | ConvertFrom-Json).actualState.transport | Should -BeExactly 'jsonrpc'
            } else {
                $LASTEXITCODE | Should -Not -Be 0
                $output | Out-String | Should -Match '150'
                $output | Out-String | Should -Match 'timed out'
            }
        } finally {
            Pop-Location
        }
    }

    It 'forwards fractional progress without changing configuration identity or counters' {
        $configuration = New-RpcConfiguration -Properties @{ notifications = $true }
        $progressPath = Join-Path $TestDrive 'progress.jsonl'
        $null = dsc --progress-format json config get -i ($configuration | ConvertTo-Json -Depth 20) 2> $progressPath
        $LASTEXITCODE | Should -Be 0
        $records = @(Get-Content $progressPath | Where-Object { $_.StartsWith('{') } | ConvertFrom-Json)
        $subordinate = @($records | Where-Object resourceProgress)
        $subordinate.Count | Should -BeGreaterThan 0
        @($records | Where-Object resourceType -EQ 'Test/JsonRpc' | Select-Object -ExpandProperty id -Unique) | Should -HaveCount 1
        $subordinate[0].resourceProgress.completed | Should -Be 0.5
        $subordinate[0].totalItems | Should -Be 2
        $subordinate[0].completedItems | Should -Be 0
        $subordinate[0].resourceName | Should -BeExactly 'first'
        $records[-1].resourceProgress | Should -BeNullOrEmpty
    }
}
