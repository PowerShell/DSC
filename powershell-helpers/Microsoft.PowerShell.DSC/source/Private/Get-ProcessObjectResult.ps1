function Get-ProcessObjectResult
{
    [CmdletBinding(SupportsShouldProcess)]
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
    # TODO: Somehow when we use input with JSON it doesn't work as expected
    $info.Arguments = $SubCommand
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true

    $proc.StartInfo = $info

    if ($PSCmdlet.ShouldProcess(("{0}", "{1}" -f $info.FileName, $info.Arguments)))
    {
        # start the process
        $proc.Start() | Out-Null

        # read stream outputs
        $stdOut = Get-ProcessOutput -Process $proc -ReadLine StandardOutput
        $stErr = Get-ProcessOutput -Process $proc -ReadLine StandardError

        # TODO: Get process output when JSON cannot be returned

        # wait for exit
        $proc.WaitForExit()

        $inputObject = New-Object -TypeName PSObject -Property ([Ordered]@{
                Executable = $info.FileName
                Arguments  = $SubCommand
                ExitCode   = $proc.ExitCode
                Output     = $stdOut
                Error      = $stErr
            })
        return $inputObject
    }
}
