# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    if ($IsWindows) {
        $identity = [System.Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = [System.Security.Principal.WindowsPrincipal]::new($identity)
        $isElevated = $principal.IsInRole([System.Security.Principal.WindowsBuiltInRole]::Administrator)

        if ($env:GITHUB_ACTION) {
            $script:currentModulePaths = $env:PSModulePath
            Write-Verbose -Message "Running in GitHub Actions" -Verbose
            # Uninstall the PSDesiredStateConfiguration module as this requires v1.1 and the build script installs it 
            Uninstall-PSResource -Name 'PSDesiredStateConfiguration' -Version 2.0.7 -ErrorAction Stop
            # Get current PSModulePath and exclude PowerShell 7 paths
            $currentPaths = $env:PSModulePath -split ';' | Where-Object { 
                $_ -notmatch 'PowerShell[\\/]7' -and 
                $_ -notmatch 'Program Files[\\/]PowerShell[\\/]' -and
                $_ -notmatch 'Documents[\\/]PowerShell[\\/]'
            }
            
            # Check if Windows PowerShell modules path exists
            $windowsPSPath = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\Modules"
            if ($windowsPSPath -notin $currentPaths) {
                $currentPaths += $windowsPSPath
            }
            
            # Update PSModulePath
            $env:PSModulePath = $currentPaths -join ';'
        }
    }
}

Describe 'PowerShell extension tests' {
    It 'Example PowerShell file should work' -Skip:(!$IsWindows -or !$isElevated) {
        $psFile = Resolve-Path -Path "$PSScriptRoot\..\..\dsc\examples\variable.dsc.ps1"
        $out = dsc -l trace config get -f $psFile 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log -Raw | Out-String)
        $out.results[0].result.actualState.Ensure | Should -Be 'Absent'
        (Get-Content -Path $TestDrive/error.log -Raw) | Should -Match "Importing file '$psFile' with extension 'Microsoft.DSC.Extension/PowerShell'"
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
        $content | Should -BeLike "*Importing file '$psFile' with extension 'Microsoft.DSC.Extension/WindowsPowerShell'*"
        $content | Should -Match "No DSC resources found in the imported modules."
    }
}

AfterAll {
    if ($IsWindows -and $env:GITHUB_ACTION) {
        Install-PSResource -Name 'PSDesiredStateConfiguration' -Version 2.0.7 -ErrorAction Stop -TrustRepository -Reinstall
    }

    Write-Verbose -Message "Restoring original PSModulePath" -Verbose
    Write-Verbose -Message ($script:currentModulePaths) -Verbose
    $env:PSModulePath = $script:currentModulePaths
}