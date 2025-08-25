# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    $runningInCI = $false
}

BeforeAll {
    $FullyQualifiedName = @()
    $FullyQualifiedName += @{ModuleName="Microsoft.PowerShell.SecretManagement";ModuleVersion="1.1.2"}
    $FullyQualifiedName += @{ModuleName="Microsoft.PowerShell.SecretStore";ModuleVersion="1.0.6"}
    foreach ($module in $FullyQualifiedName) {
        if (-not (Get-Module -ListAvailable -FullyQualifiedName $module)) {
            Save-PSResource -Name $module.ModuleName -Version $module.ModuleVersion -Path $TestDrive -Repository PSGallery -TrustRepository
        }
    }

    $env:PSModulePath += [System.IO.Path]::PathSeparator + $TestDrive
}

Describe 'Tests for PowerShell Secret Management' -Skip:($runningInCI) {
    It 'Should get secret from default store' {
        # Instead of doing it in the BeforeAll block, reset the store here as we know we are running in the CI
        Reset-SecretStore -Password (ConvertTo-SecureString -AsPlainText -String 'P@ssw0rd' -Force) -Force
        Register-SecretVault -Name 'VaultA' -ModuleName 'Microsoft.PowerShell.SecretStore' -DefaultVault
        Set-Secret -Name TestSecret -Secret "Super@SecretPassword"

        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('TestSecret')]"
'@
        $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/error.log)
        $out.results.Count | Should -Be 1
        $out.results[0].result.actualState.Output | Should -BeExactly 'Super@SecretPassword'
    }
}