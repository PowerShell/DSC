# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentVariableList get operation' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentVariableList'
        $testName = "DSC_Environment_Get_$([guid]::NewGuid().ToString('N'))"
        $testValue = 'C:\DSC\First;C:\DSC\Second'
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName -Value $testValue -Type String
    }

    AfterAll {
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name $testName -ErrorAction Ignore
    }

    It 'Gets a CurrentUser variable using the default scope' {
        $json = @{
            environmentVariables = @(
                @{ name = $testName }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).actualState.environmentVariables[0]

        $result.scope | Should -BeExactly 'currentUser'
        $result.name | Should -BeExactly $testName
        $result.value | Should -BeExactly $testValue
        $result._exist | Should -BeTrue
        $result.PSObject.Properties.Name | Should -Not -Contain 'pathAction'
    }

    It 'Gets a variable as pathValue when pathValue is requested' {
        $json = @{
            environmentVariables = @(
                @{
                    name      = $testName
                    pathValue = @()
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).actualState.environmentVariables[0]

        ($result.pathValue | ConvertTo-Json -Compress) |
            Should -BeExactly '["C:\\DSC\\First","C:\\DSC\\Second"]'
        $result.PSObject.Properties.Name | Should -Not -Contain 'value'
    }

    It 'Returns _exist false for a missing variable' {
        $missingName = "DSC_Environment_Missing_$([guid]::NewGuid().ToString('N'))"
        $json = @{
            environmentVariables = @(
                @{ name = $missingName }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).actualState.environmentVariables[0]

        $result.name | Should -BeExactly $missingName
        $result._exist | Should -BeFalse
        $result.PSObject.Properties.Name | Should -Not -Contain 'value'
    }

    It 'Gets multiple variables in input order' {
        $missingName = "DSC_Environment_Missing_$([guid]::NewGuid().ToString('N'))"
        $json = @{
            environmentVariables = @(
                @{ name = $testName }
                @{ name = $missingName }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).actualState.environmentVariables

        $result.Count | Should -Be 2
        $result[0].name | Should -BeExactly $testName
        $result[1].name | Should -BeExactly $missingName
    }
}
