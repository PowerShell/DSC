function New-SubCommand 
{
    [CmdletBinding()]
    [OutputType([System.Text.StringBuilder])]
    Param 
    (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [System.String]
        $SubCommand
    )

    $stringBuilder = New-Object -TypeName System.Text.StringBuilder -ArgumentList $SubCommand

    return $stringBuilder
}