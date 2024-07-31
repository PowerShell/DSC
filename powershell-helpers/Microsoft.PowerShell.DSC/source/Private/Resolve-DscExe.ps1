function Resolve-DscExe 
{
    [OutputType([System.String])]
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $false)]
        [ValidateNotNullOrEmpty()]
        [System.IO.FileInfo]
        $Path,

        [Parameter(Mandatory = $false)]
        [ValidateSet('Machine', 'User', 'Process')]
        [System.String]
        $Scope = 'Machine'
    )

    if ($PSBoundParameters.ContainsKey('Path') -and (-not (Test-Path $Path -PathType Leaf)))
    {
        Throw "No file found at path '$Path'. Please specify the file path to 'dsc.exe'"
    }

    if ($IsWindows)
    {
        $exe = Join-Path -Path (Get-EnvironmentVariable -Name 'DSC_RESOURCE_PATH' -Scope $Scope -Expanded) -ChildPath 'dsc.exe'
        if (Test-Path $exe)
        {
            return $exe
        }

        $exe = (Get-Command dsc -ErrorAction SilentlyContinue).Source
        if (-not $exe)
        {
            Throw "Could not locate 'dsc.exe'. Please make sure it can be found through the PATH or DSC_RESOURCE_PATH environment variable."
        }

        return $exe
    }
}