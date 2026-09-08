# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentVariable test operation' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentVariable'
        $testName = "DSC_Environment_Single_Test_$([guid]::NewGuid().ToString('N'))"
    }

    AfterEach {
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name $testName -ErrorAction Ignore
    }

    It 'Reports a matching scalar value in desired state' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName -Value 'expected' -Type String
        $json = @{
            name  = $testName
            value = 'expected'
        } | ConvertTo-Json -Compress

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = $out | ConvertFrom-Json

        $result.inDesiredState | Should -BeTrue
        $result.actualState.name | Should -BeExactly $testName
        $result.actualState.PSObject.Properties.Name | Should -Not -Contain 'environmentVariables'
    }

    It 'Honors pathAction for one environment variable' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName `
            -Value 'C:\Existing;C:\New' -Type String
        $json = @{
            name       = $testName
            pathValue  = @('C:\New')
            pathAction = 'append'
        } | ConvertTo-Json -Compress

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }
}
