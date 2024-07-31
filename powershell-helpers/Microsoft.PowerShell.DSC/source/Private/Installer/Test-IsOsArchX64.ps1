function Test-IsOsArchX64
{
    if ($PSVersionTable.PSVersion.Major -lt 6)
    {
        return (Get-CimInstance -ClassName Win32_OperatingSystem).OSArchitecture -match '64'
    }
        
    return [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq [System.Runtime.InteropServices.Architecture]::X64
}