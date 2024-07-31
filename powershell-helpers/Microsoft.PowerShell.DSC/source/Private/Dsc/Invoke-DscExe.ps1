function Invoke-DscExe 
{
    [CmdletBinding(SupportsShouldProcess)]
    Param 
    (
        [Parameter(Mandatory = $false)]
        [System.String]
        $SubCommand = 'resource list'
    )

    begin 
    {
        Write-Verbose ("Starting: {0}" -f $MyInvocation.MyCommand.Name)
    }

    process 
    {
        $inputObject = Get-ProcessObjectResult -SubCommand $SubCommand
    }
    end
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
        return $inputObject
    }
}