# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentVariable set operation' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentVariable'
        $testName = "DSC_Environment_Single_Set_$([guid]::NewGuid().ToString('N'))"
    }

    AfterEach {
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name $testName -ErrorAction Ignore
    }

    It 'Sets one scalar environment variable without a list envelope' {
        $json = @{
            name  = $testName
            value = 'single value'
        } | ConvertTo-Json -Compress

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState

        $result.name | Should -BeExactly $testName
        $result.value | Should -BeExactly 'single value'
        $result.PSObject.Properties.Name | Should -Not -Contain 'environmentVariables'
        (Get-ItemPropertyValue -Path 'HKCU:\Environment' -Name $testName) |
            Should -BeExactly 'single value'
    }

    It 'Removes one environment variable with _exist false' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName -Value 'remove me' -Type String
        $json = @{
            name   = $testName
            _exist = $false
        } | ConvertTo-Json -Compress

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).afterState._exist | Should -BeFalse
        { Get-ItemPropertyValue -Path 'HKCU:\Environment' -Name $testName -ErrorAction Stop } |
            Should -Throw
    }
}
