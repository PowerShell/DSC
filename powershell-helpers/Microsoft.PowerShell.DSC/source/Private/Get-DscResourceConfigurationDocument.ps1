function Get-DscResourceConfigurationDocument 
{
    [CmdletBinding(DefaultParameterSetName = 'ByInput')]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [Alias('Name')]
        [System.String]
        $ResourceName,

        [Parameter(Mandatory = $false, ParameterSetName = 'ByPath')]
        [Alias('Path')]
        [System.IO.FileInfo]
        $ResourcePath,

        [Parameter(Mandatory = $false, ParameterSetName = 'ByInput')]
        [hashtable]
        $ResourceInput = @{}
    )

    begin 
    {
        $commandName = $MyInvocation.MyCommand.Name 
        Write-Verbose ("Starting: {0}" -f $commandName)

        # get data
        Write-Verbose -Message "Gathering command data for '$commandName'"
        $data = Get-DscCommandData -CommandName $commandName -IncludeProperties -ResourceName $ResourceName -Operation $commandName.Split("-")[0]

        Write-Verbose -Message "Building sub command with:"
        Write-Verbose -Message ("{0}{1}" -f $data.SubCommand, " --resource $resourceName")
        $subCommand = New-SubCommand -SubCommand ("{0}{1}" -f $data.SubCommand, " --resource $resourceName")
    }

    process 
    {
        Build-DscPathBuilder -Data $data -SubCommand $SubCommand -ResourceName $ResourceName -ResourcePath $ResourcePath -ResourceInput $ResourceInput

        $inputObject = Invoke-DscExe -SubCommand $subCommand.ToString()
    }
    end
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
        return $inputObject
    }
}