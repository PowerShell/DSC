# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$ErrorActionPreference = 'Stop'
[Console]::InputEncoding = [Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)

function Write-RpcMessage {
    param([hashtable]$Message)
    [Console]::Out.WriteLine(($Message | ConvertTo-Json -Depth 30 -Compress))
    [Console]::Out.Flush()
}

function Read-RpcMessage {
    $line = [Console]::In.ReadLine()
    if ($null -eq $line) {
        throw 'Unexpected end of JSON-RPC input'
    }
    $line | ConvertFrom-Json -AsHashtable
}

function Add-RpcEvent {
    param([string]$Role, [string]$Event, [hashtable]$Details = @{})
    if ($env:DSC_TEST_EXTENSION_RPC_LOG) {
        $entry = @{ pid = $PID; role = $Role; event = $Event; details = $Details }
        [IO.File]::AppendAllText(
            $env:DSC_TEST_EXTENSION_RPC_LOG,
            ($entry | ConvertTo-Json -Depth 10 -Compress) + "`n"
        )
    }
}

function Get-RpcSettings {
    Get-Content -LiteralPath $env:DSC_TEST_EXTENSION_RPC_SETTINGS -Raw |
        ConvertFrom-Json -AsHashtable
}
