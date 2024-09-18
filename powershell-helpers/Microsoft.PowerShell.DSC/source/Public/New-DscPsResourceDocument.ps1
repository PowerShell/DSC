function New-DscPsResourceDocument
{
    [CmdletBinding(DefaultParameterSetName = 'Required')]
    param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $ResourceName,

        [System.Management.Automation.SwitchParameter]
        $OnlyRequired,
        
        [System.Management.Automation.SwitchParameter]
        $IncludeProperties
    )

    begin 
    {
        $commandName = $MyInvocation.MyCommand.Name 
        Write-Verbose ("Starting: {0}" -f $commandName)
    }

    process 
    {
        $cacheFilePath = Get-DscPsCacheRefreshPath 

        if (-not $cacheFilePath)
        {
            # TODO: It can be replaced after GitHub issue is solved to call the command directly from dsc.exe
            Throw "Please execute 'Invoke-DscCacheRefresh' from the 'psDscAdapter.psm1' module file" 
        }

        $json = Get-Content $cacheFilePath | ConvertFrom-Json

        # try to find the object in the cache and always filter only one result
        $resourceObject = $json.ResourceCache.DscResourceInfo | Where-Object {$_.Name -eq $ResourceName} | Select-Object -First 1

        if (-not $resourceObject)
        {
            Throw "No resource found with name: '$ResourceName'. Please make sure you have installed the DSC PowerShell module that exports this resource."
        }

        $propArgs = @{
            Properties = $resourceObject.Properties
        }

        if ($OnlyRequired)
        {
            $propArgs.Add('Required', $true)
        }

        return (Get-DscPsCacheProperties @propArgs)
    }

    end 
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
    }
}