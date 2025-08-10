# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)
    }
}

Describe 'PowerShell extension tests' {
    It 'Example PowerShell file should work' -Skip:(!$IsWindows -or !$isElevated) {
        $psFile = Resolve-Path -Path "$PSScriptRoot\..\..\dsc\examples\variable.dsc.ps1"
        $out = dsc -l trace config get -f $psFile 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $out.results[0].result.actualState.Ensure | Should -Be 'Absent'
        $psFile = $psFile.ToString().Replace('\', '\\')
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Match "Importing file '$psFile' with extension 'Microsoft.DSC.Extension/WindowsPowerShell'"
    }

    It 'Invalid PowerShell configuration document file returns error' -Skip:(!$IsWindows) {
        $psFile = "$TestDrive/invalid.ps1"
        Set-Content -Path $psFile -Value @"
configuration InvalidConfiguration {
    Import-DscResource -ModuleName InvalidModule
    Node localhost
    {
        Test Invalid {
            Name = 'InvalidTest'
            Ensure = 'Present'
        }
    }
}
"@
        dsc -l trace config get -f $psFile 2>$TestDrive/error.log 
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $content = (Get-Content -Path $TestDrive/error.log -Raw)
        $psFile = $psFile.ToString().Replace('\', '\\')
        $content | Should -Match "Importing file '$psFile' with extension 'Microsoft.DSC.Extension/WindowsPowerShell'"
        $content | Should -Match "No DSC resources found in the imported modules."
    }
}
