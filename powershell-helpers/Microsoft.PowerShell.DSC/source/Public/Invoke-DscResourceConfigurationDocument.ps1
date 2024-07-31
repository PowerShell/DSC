function Invoke-DscResourceConfigurationDocument 
{
    [CmdletBinding(SupportsShouldProcess)]
    param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $ResourceName,

        [Parameter(Mandatory = $false)]
        [ValidateSet('Get', 'Set', 'Test')]
        [System.String]
        $Operation = 'Get',

        [Parameter(Mandatory = $false)]
        [AllowNull()]
        [Alias('Path')]
        [System.IO.FileInfo]
        $ResourcePath,

        [Parameter(Mandatory = $false)]
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
        # build arguments for each function
        $arguments = @{ Name = $ResourceName} 
        if ($ResourcePath)
        {
            $arguments.Add('ResourcePath', $ResourcePath)
        }

        if ($ResourceInput)
        {
            $arguments.Add('ResourceInput', $ResourceInput)
        }
        else
        {
            $arguments.Add('ResourceInput', @{})
        }

        switch ($Operation)
        {
            'Get' 
            {
                $inputObject = Get-DscResourceConfigurationDocument @arguments
            }
            'Set'
            {
                $inputObject = Set-DscResourceConfigurationDocument @arguments
            }
            'Test'
            {   
                $inputobject = Test-DscResourceConfigurationDocument @arguments
            }
            default {$inputObject = @{}}
        }

        return $inputObject
    }

    end 
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
    }
}