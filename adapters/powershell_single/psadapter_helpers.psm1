# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

function Write-DscTrace {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Level,
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message,
        [switch]$Now
    )

    $trace = @{$Level.ToLower() = $Message } | ConvertTo-Json -Compress

    if ($Now) {
        $host.ui.WriteErrorLine($trace)
    } else {
        $traceQueue.Enqueue($trace)
    }
}

function Invoke-Script {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Script
    )

    $ps = [PowerShell]::Create().AddScript({
        $DebugPreference = 'Continue'
        $VerbosePreference = 'Continue'
        $ErrorActionPreference = 'Stop'
    }).AddStatement().AddScript($script)

    $traceQueue = [System.Collections.Concurrent.ConcurrentQueue[object]]::new()

    $null = Register-ObjectEvent -InputObject $ps.Streams.Error -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        # convert error to string since it's an ErrorRecord
        $traceQueue.Enqueue((@{ error = [string]$EventArgs.ItemAdded } | ConvertTo-Json -Compress))
    }
    $null = Register-ObjectEvent -InputObject $ps.Streams.Warning -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        $traceQueue.Enqueue((@{ warn = $EventArgs.ItemAdded.Message } | ConvertTo-Json -Compress))
    }
    $null = Register-ObjectEvent -InputObject $ps.Streams.Information -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        if ($null -ne $EventArgs.ItemAdded.MessageData) {
            if ($EventArgs.ItemAdded.Tags -contains 'PSHOST') {
                $traceQueue.Enqueue((@{ info = $EventArgs.ItemAdded.MessageData.ToString() } | ConvertTo-Json -Compress))
            } else {
                $traceQueue.Enqueue((@{ trace = $EventArgs.ItemAdded.MessageData.ToString() } | ConvertTo-Json -Compress))
            }
            return
        }
    }
    $null = Register-ObjectEvent -InputObject $ps.Streams.Verbose -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        $traceQueue.Enqueue((@{ info = $EventArgs.ItemAdded.Message } | ConvertTo-Json -Compress))
    }
    $null = Register-ObjectEvent -InputObject $ps.Streams.Debug -EventName DataAdding -MessageData $traceQueue -Action {
        $traceQueue = $Event.MessageData
        $traceQueue.Enqueue((@{ debug = $EventArgs.ItemAdded.Message } | ConvertTo-Json -Compress))
    }
    $outputObjects = [System.Collections.Generic.List[Object]]::new()

    function Write-TraceQueue() {
        $trace = $null
        while (!$traceQueue.IsEmpty) {
            if ($traceQueue.TryDequeue([ref] $trace)) {
                $host.ui.WriteErrorLine($trace)
            }
        }
    }

    try {
        $asyncResult = $ps.BeginInvoke()
        while (-not $asyncResult.IsCompleted) {
            Write-TraceQueue
        
            Start-Sleep -Milliseconds 100
        }
        $outputCollection = $ps.EndInvoke($asyncResult)
        Write-TraceQueue


        if ($ps.HadErrors) {
            # If there are any errors, we will exit with an error code
            Write-DscTrace -Now -Level Error -Message 'Errors occurred during script execution.'
            exit 1
        }

        foreach ($output in $outputCollection) {
            $outputObjects.Add($output)
        }
    }
    catch {
        Write-DscTrace -Now -Level Error -Message $_
        exit 1
    }
    finally {
        $ps.Dispose()
        Get-EventSubscriber | Unregister-Event
    }
}
