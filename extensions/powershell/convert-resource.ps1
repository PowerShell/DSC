[CmdletBinding()]
param (
    [Parameter(ValueFromPipeline = $true)]
    [string[]]$stringInput
)

begin {
    $lines = [System.Collections.Generic.List[string]]::new()
    
    if ($PSVersionTable.PSEdition -ne 'Core') {
        # Remove all PowerShell paths
        $env:PSModulePath = ($env:PSModulePath -split ';' | Where-Object { 
            $_ -notmatch 'PowerShell[\\/]7' -and 
            $_ -notmatch 'Program Files[\\/]PowerShell[\\/]' -and
            $_ -notmatch 'Documents[\\/]PowerShell[\\/]'
        }) -join ';'

        # Make sure the default path is Windows PowerShell is included
        $winPsPath = "$env:windir\System32\WindowsPowerShell\v1.0\Modules"
        if ($env:PSModulePath -notmatch [regex]::Escape($winPsPath)) {
            # Separator is already at the end
            $env:PSModulePath = $env:PSModulePath + $winPsPath
        }
    }

    $scriptModule = Import-Module "$PSScriptRoot/convertDscResource.psd1" -Force -PassThru -WarningAction SilentlyContinue -ErrorAction Stop
}

process {
    foreach ($line in $stringInput) {
        $lines.Add($line)  
    }
}

end {
    if ($lines.Count -ne 0) {
        $result = $scriptModule.invoke( { param($lines) Build-DscConfigDocument -Content $lines }, ($lines | Out-String) )

        return ($result | ConvertTo-Json -Depth 10 -Compress)
    }
}