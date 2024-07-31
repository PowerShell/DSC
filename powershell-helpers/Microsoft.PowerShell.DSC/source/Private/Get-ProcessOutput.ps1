function Get-ProcessObjectResult {
    param
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $SubCommand
    )

    # TODO: Add global settings for resolving
    $dscExePath = Resolve-DscExe 

    # use System.Diagnostics.Process instead of & or Invoke-Expression
    $proc = [System.Diagnostics.Process]::new()

    # create the starter information
    $info = New-Object System.Diagnostics.ProcessStartInfo
    $info.FileName = $dscExePath
    $info.Arguments = $SubCommand
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true

    $proc.StartInfo = $info

    # start process
    $proc.Start() | Out-Null

    # read stream outputs
    $stdOut = Get-ProcessOutput -Process $proc -ReadLine StandardOutput
    $stErr = Get-ProcessOutput -Process $proc -ReadLine StandardError

    # wait for exit
    $proc.WaitForExit()

    $inputObject = New-Object -TypeName PSObject -Property ([Ordered]@{
        Executable = $info.FileName
        Arguments = $SubCommand
        ExitCode = $proc.ExitCode
        Output = $stdOut
        Error = $stErr
    })

    return $inputObject
}

function Get-ProcessOutput 
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.Diagnostics.Process]
        $Process,

        [Parameter(Mandatory = $false)]
        [ValidateSet('StandardOutput', 'StandardError')]
        [System.String]
        $ReadLine = 'StandardOutput'
    )

    # TODO: Can determine if classes can be created to have more strong-typed
    $output = [System.Collections.Generic.List[object]]::new()

    do
    {
        $line = $Process.$ReadLine.ReadLine()

        if ($line)
        {
            try 
            {
                $jsonOutput = $line | ConvertFrom-Json    

                # add to output
                $output.Add($jsonOutput)
            }
            catch 
            {
                $msg = "Could not convert '$line' to JSON."
                Write-Debug -Message $msg
            }
        }
        else 
        {
            break
        }
    } while ($true)

    return $output
}