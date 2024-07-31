function Get-DscPsCacheProperties 
{
    [CmdletBinding()]
    [OutputType([System.Collections.Hashtable])]
    param 
    (
        [Parameter(Mandatory = $true)]
        [object]
        $Properties,

        [System.Management.Automation.SwitchParameter]
        $Required
    )

    if ($Required)
    {
        $properties = $properties | Where-Object {$_.IsMandatory -eq $true }
    }

    $inputObject = $properties | ForEach-Object {
        @{$_.Name = "<$($_.Name)>"}
    }

    return $inputObject
}