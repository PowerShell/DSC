# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

param([string]$ValueOverride)

. "$PSScriptRoot/../protocol.ps1"
$initialized = $false
$shuttingDown = $false
Add-RpcEvent -Role secret -Event start

:requests while ($true) {
    $request = Read-RpcMessage
    $method = $request.method
    $parameters = $request.params
    Add-RpcEvent -Role secret -Event $method -Details @{
        id = $request.id
        name = $parameters.name
        vault = $parameters.vault
        hostServices = $parameters.hostServices
        dscVersion = $parameters.dscVersion
        reuseForbidden = $env:DSC_RESOURCE_REUSE_FORBIDDEN
    }
    switch ($method) {
        initialize {
            if ($initialized) { throw 'Already initialized' }
            $initialized = $true
            $result = @{ protocolVersion = 1; operations = @('secret') }
        }
        dsc_invoke {
            if (-not $initialized -or $shuttingDown -or $parameters.operation -ne 'secret') {
                throw 'Invalid secret invocation'
            }
            switch ($env:DSC_TEST_SECRET_RPC_FAULT) {
                timeout { Start-Sleep -Seconds 30 }
                error {
                    [Console]::Error.WriteLine('{"error":"test-runtime-token test-runtime-password"}')
                    Write-RpcMessage @{
                        jsonrpc = '2.0'; method = 'dsc_trace'
                        params = @{ requestId = $request.id; level = 'error'; message = 'test-runtime-password' }
                    }
                    Write-RpcMessage @{
                        jsonrpc = '2.0'; id = $request.id
                        error = @{ code = -32000; message = 'test-runtime-token' }
                    }
                    continue requests
                }
                reverse {
                    Write-RpcMessage @{
                        jsonrpc = '2.0'; id = 'provider-callback'; method = 'secret.get'
                        params = @{ name = 'recursive-secret' }
                    }
                    $response = Read-RpcMessage
                    if ($response.id -ne 'provider-callback' -or $response.error.code -ne -32601) {
                        throw 'Secret providers must not recursively resolve secrets'
                    }
                    Add-RpcEvent -Role secret -Event callbackRejected
                }
            }
            $settings = Get-RpcSettings
            $key = "$($parameters.vault)/$($parameters.name)"
            $result = $settings.secrets[$key]
            if ($ValueOverride -and $null -ne $result) {
                $result = @{ secureString = $ValueOverride }
            }
            if ($env:DSC_TEST_SECRET_RPC_FAULT -eq 'invalid') {
                $result = @{ unexpected = 'not a secret result' }
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
