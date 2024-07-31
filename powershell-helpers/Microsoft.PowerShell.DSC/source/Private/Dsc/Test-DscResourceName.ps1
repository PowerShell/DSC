function Test-DscResourceName 
{
    [CmdletBinding()]
    [OutputType([System.Boolean])]
    Param
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $ResourceName,

        [Parameter(Mandatory = $false)]
        [AllowNull()]
        [System.String[]]
        $Resources
    )

    if ($ResourceName -in $Resources)
    {
        return $true
    }

    return $false
}