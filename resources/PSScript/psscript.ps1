# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateSet('Get', 'Set', 'Test')]
    [string]$Operation,
    [Parameter(Mandatory = $true, Position = 1, ValueFromPipeline = $true)]
    [string]$jsonInput
)

$traceQueue = [System.Collections.Concurrent.ConcurrentQueue[object]]::new()

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
}

if ($null -eq $script) {
    Write-DscTrace -Now -Level Info -Message "No script found for operation '$Operation'."
    if ($Operation -eq 'Test') {
        # if not implemented, we return it's in desired state
        @{ _inDesiredState = $true } | ConvertTo-Json -Compress
        exit 0
    }

    # write an empty json object to stdout
    '{}'
    exit 0
}

# use AST to see if script has param block, if any errors exit with error message
$errors = $null
$tokens = $null
$ast = [System.Management.Automation.Language.Parser]::ParseInput($script, [ref]$tokens, [ref]$errors)
if ($errors.Count -gt 0) {
    $errorMessage = $errors | ForEach-Object { $_.ToString() }
    Write-DscTrace -Now -Level Error -Message "Script has syntax errors: $errorMessage"
    exit 3
}

$paramName = if ($ast.ParamBlock -ne $null) {
    # make sure it only specifies one parameter and get the name of that parameter
    if ($ast.ParamBlock.Parameters.Count -ne 1) {
        Write-DscTrace -Now -Level Error -Message 'Script must have exactly one parameter.'
        exit 3
    }
    $ast.ParamBlock.Parameters[0].Name.VariablePath.UserPath
} else {
    $null
}

$ps = [PowerShell]::Create().AddScript({
    $DebugPreference = 'Continue'
    $VerbosePreference = 'Continue'
    $ErrorActionPreference = 'Stop'
}).AddScript($script)

if ($null -ne $scriptObject.input) {
    if ($null -eq $paramName) {
        Write-DscTrace -Now -Level Error -Message 'Input was provided but script does not have a parameter to accept input.'
        exit 3
    }
    $null = $ps.AddParameter($paramName, $scriptObject.input)
} elseif ($null -ne $paramName) {
    Write-DscTrace -Now -Level Error -Message "Script has a parameter '$paramName' but no input was provided."
    exit 3
}

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
    Write-DscTrace -Level Trace -Message $sender.MessageData.ToString()
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

function write-traces() {
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
        write-traces
        Start-Sleep -Milliseconds 100
    }
    $outputCollection = $ps.EndInvoke($asyncResult)
    write-traces

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
    Write-DscTrace -Now -Level Error -Message $_.Exception.Message
    exit 1
}
finally {
    $ps.Dispose()
}

# Test should return a single boolean value indicating if in the desired state
if ($Operation -eq 'Test') {
    if ($outputObjects.Count -eq 1 -and $outputObjects[0] -is [bool]) {
        @{ _inDesiredState = $outputObjects[0] } | ConvertTo-Json -Compress
    } else {
        Write-DscTrace -Now -Level Error -Message 'Test operation did not return a single boolean value.'
        exit 1
    }
} else {
    @{ output = $outputObjects } | ConvertTo-Json -Compress -Depth 10
}
