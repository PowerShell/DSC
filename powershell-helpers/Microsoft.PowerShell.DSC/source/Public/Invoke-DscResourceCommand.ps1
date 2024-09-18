function Invoke-DscResourceCommand 
{
    [CmdletBinding(SupportsShouldProcess, DefaultParameterSetName = '__AllParameterSets')]
    param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $ResourceName,

        [Parameter(Mandatory = $false)]
        [ValidateSet('Get', 'Set', 'Test')]
        [System.String]
        $Operation = 'Get',

        [Parameter(Mandatory = $false, ParameterSetName = 'ByPath')]
        [AllowNull()]
        [Alias('Path')]
        [System.IO.FileInfo]
        $ResourcePath,

        [Parameter(Mandatory = $false, ParameterSetName = 'ByInput')]
        [AllowNull()]
        [hashtable]
        $ResourceInput
    )

    begin 
    {
        $commandName = $MyInvocation.MyCommand.Name 
        Write-Verbose ("Starting: {0}" -f $commandName)
    }

    process 
    {
        $arguments = @{ResourceName = $ResourceName }
        # get argument data
        switch ($PSCmdlet.ParameterSetName)
        {
            'ResourcePath' { $arguments.Add('ResourcePath', $ResourcePath) }
            'ResourceInput' { $arguments.Add('ResourceInput', $ResourceInput) }
            default { $arguments.Add('ResourceInput', @{}) }
        }

        # go through operations
        switch ($Operation)
        {
            'Get' 
            {
                $inputObject = Get-DscResourceCommand @arguments
            }
            'Set'
            {
                $inputObject = Set-DscResourceCommand @arguments
            }
            'Test'
            {   
                $inputobject = Test-DscResourceCommand @arguments
            }
            default { $inputObject = @{} }
        }

        return $inputObject
    }

    end 
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
    }
}