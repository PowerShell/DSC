# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentVariable get operation' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentVariable'
        $testName = "DSC_Environment_Single_Get_$([guid]::NewGuid().ToString('N'))"
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName -Value 'single value' -Type String
    }

    AfterAll {
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name $testName -ErrorAction Ignore
    }

    It 'Gets one environment variable without a list envelope' {
        $json = @{ name = $testName } | ConvertTo-Json -Compress

        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).actualState

        $result.scope | Should -BeExactly 'currentUser'
        $result.name | Should -BeExactly $testName
        $result.value | Should -BeExactly 'single value'
        $result._exist | Should -BeTrue
        $result.PSObject.Properties.Name | Should -Not -Contain 'environmentVariables'
    }

    It 'Returns _exist false for a missing variable' {
        $json = @{ name = "${testName}_Missing" } | ConvertTo-Json -Compress

        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).actualState

        $result._exist | Should -BeFalse
        $result.PSObject.Properties.Name | Should -Not -Contain 'value'
    }
}
