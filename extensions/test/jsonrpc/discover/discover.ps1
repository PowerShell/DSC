# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

param([switch]$ResourceMode)

. "$PSScriptRoot/../protocol.ps1"
$initialized = $false
$shuttingDown = $false
Add-RpcEvent -Role discover -Event start

:requests while ($true) {
    $request = Read-RpcMessage
    Add-RpcEvent -Role discover -Event $request.method -Details @{
        id = $request.id; hostServices = $request.params.hostServices
    }
    switch ($request.method) {
        initialize {
            if ($initialized -or $request.params.hostServices -notcontains 'secret.get') {
                throw 'Missing host secret service'
            }
            $initialized = $true
            $operations = @(if ($ResourceMode) { 'get' } else { 'discover' })
            $result = @{ protocolVersion = 1; operations = $operations }
        }
        dsc_invoke {
            $operation = if ($ResourceMode) { 'get' } else { 'discover' }
            if (-not $initialized -or $shuttingDown -or $request.params.operation -ne $operation) {
                throw 'Invalid discovery invocation'
            }
            if (-not $ResourceMode -and $request.params.extensions -notcontains '.dsc.resource.json') {
                throw 'Missing recognized manifest extensions'
            }
            $settings = Get-RpcSettings
            $index = 0
            $callbacks = @(
                foreach ($reference in $settings.requests) {
                    $index++
                    $id = if ($settings.numericIds) { $index } else { "secret-$index" }
                    $method = if ($reference.method) { $reference.method } else { 'secret.get' }
                    $parameters = @{ name = $reference.name }
                    if ($reference.ContainsKey('vault')) { $parameters.vault = $reference.vault }
                    if ($reference.ContainsKey('params')) { $parameters = $reference.params }
                    $callback = @{ jsonrpc = '2.0'; id = $id; method = $method; params = $parameters }
                    if ($settings.malformedId) { $callback.id = $null }
                    Write-RpcMessage -Message $callback
                    @{ id = $id; reference = $reference }
                }
            )
            $failure = $null
            foreach ($callback in $callbacks) {
                $response = Read-RpcMessage
                if ($response.id -ne $callback.id) { throw 'Mismatched callback response ID' }
                $reference = $callback.reference
                if ($reference.expectedError) {
                    if ($response.error.code -ne $reference.expectedError) {
                        throw 'Unexpected callback error'
                    }
                } elseif ($response.ContainsKey('error')) {
                    $failure = "Secret callback failed ($($response.error.code))"
                } elseif (($response.result | ConvertTo-Json -Depth 20 -Compress) -cne
                          ($reference.expected | ConvertTo-Json -Depth 20 -Compress)) {
                    throw 'Secret callback returned an unexpected value'
                }
                Add-RpcEvent -Role discover -Event callback -Details @{
                    id = $response.id; code = $response.error.code
                }
            }
            if ($failure) {
                Write-RpcMessage @{
                    jsonrpc = '2.0'; id = $request.id
                    error = @{ code = -32002; message = $failure }
                }
                continue requests
            }
            if ($ResourceMode) {
                $result = @{ authenticated = $true }
            } else {
                $result = @(
                    @{
                        manifestContent = @{
                            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                            type = 'Test/SecretDiscovered'
                            version = '0.1.0'
                            get = @{
                                executable = 'pwsh'
                                args = @('-NoProfile', '-NonInteractive', '-Command', '''{"authenticated":true}''')
                            }
                            schema = @{
                                embedded = @{
                                    type = 'object'
                                    properties = @{ authenticated = @{ type = 'boolean' } }
                                }
                            }
                        }
                    }
                )
                if ($settings.registry) {
                    $result[0].registry = $settings.registry
                }
                if ($settings.manifest) {
                    $result[0].manifestContent = $settings.manifest
                }
                if ($settings.additionalManifest) {
                    $result += @{
                        manifestContent = $settings.additionalManifest
                        registry = $settings.registry
                    }
                }
                if ($settings.ContainsKey('results')) {
                    $result = $settings.results
                }
            }
        }
        shutdown {
            $shuttingDown = $true
            $result = $null
        }
        exit {
            if (-not $shuttingDown) { throw 'Exit before shutdown' }
            exit 0
        }
        default { throw 'Unknown JSON-RPC method' }
    }
    Write-RpcMessage @{ jsonrpc = '2.0'; id = $request.id; result = $result }
}
