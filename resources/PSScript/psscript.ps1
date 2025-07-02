# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('Get', 'Set', 'Test', 'Export')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true)]
    [string]$jsonInput = '@{}'
)

$traceQueue = [System.Collections.Queue]::new()

function Write-DscTrace {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Level,
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{$Level.ToLower() = $Message } | ConvertTo-Json -Compress
    $traceQueue.Enqueue($trace)
}

$scriptObject = $jsonInput | ConvertFrom-Json

$script = switch ($Operation) {
    'Get' {
        $scriptObject.GetScript
    }
    'Set' {
        $scriptObject.SetScript
    }
    'Test' {
        $scriptObject.TestScript
    }
    'Export' {
        $scriptObject.ExportScript
    }
}

if ($null -eq $script) {
    Write-DscTrace -Level Info -Message "No script found for operation '$Operation'."
    exit 0
}

$ps = [PowerShell]::Create().AddScript({
    $DebugPreference = 'Continue'
    $VerbosePreference = 'Continue'
    $ErrorActionPreference = 'Stop'
}).AddScript($script)
$ps.Streams.Error.add_DataAdded({
    param($sender, $args)
    Write-DscTrace -Level Error -Message $sender.Message
})
$ps.Streams.Warning.add_DataAdded({
    param($sender, $args)
    Write-DscTrace -Level Warn -Message $sender.Message
})
$ps.Streams.Information.add_DataAdded({
    param($sender, $args)
    Write-DscTrace -Level Trace -Message $sender.Message
})
$ps.Streams.Verbose.add_DataAdded({
    param($sender, $args)
    Write-DscTrace -Level Info -Message $sender.Message
})
$ps.Streams.Debug.add_DataAdded({
    param($sender, $args)
    Write-DscTrace -Level Debug -Message $sender.Message
})
$outputObjects = [System.Collections.Generic.List[Object]]::new()

try {
    $asyncResult = $ps.BeginInvoke()
    while (-not $asyncResult.IsCompleted) {
        While ($traceQueue.Count -gt 0) {
            $trace = $traceQueue.Dequeue()
            $host.ui.WriteErrorLine($trace)
        }
    }
    $outputCollection = $ps.EndInvoke($asyncResult)
    foreach ($output in $outputCollection) {
        $outputObjects.Add($output)
    }
}
catch {
    Write-DscTrace -Level Error -Message $_.Exception.Message
    exit 1
}
finally {
    $ps.Dispose()
}

if ($ps.HadErrors) {
    # If there are any errors, we will exit with an error code
    Write-DscTrace -Level Error -Message 'Errors occurred during script execution.'
    exit 1
}

While ($traceQueue.Count -gt 0) {
    $trace = $traceQueue.Dequeue()
    $host.ui.WriteErrorLine($trace)
}

@{
    output = $outputObjects
} | ConvertTo-Json -Compress -Depth 10
