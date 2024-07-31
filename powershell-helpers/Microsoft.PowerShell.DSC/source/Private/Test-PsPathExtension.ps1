function Test-PsPathExtension
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Path
    )

    $res = $true

    if (-not (Test-Path $Path))
    {
        $res = $false
    }

    if (([System.IO.Path]::GetExtension($Path) -ne ".ps1"))
    {
        $res = $false
    }

    return $res
}