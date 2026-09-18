# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentPathVariable operations' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentPathVariable'
        $testName = "DSC_Environment_Path_$([guid]::NewGuid().ToString('N'))"
    }

    AfterEach {
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name $testName -ErrorAction Ignore
    }

    It 'Gets entries using the default delimiter' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName `
            -Value 'C:\One;C:\Two' -Type String
        $json = @{ name = $testName } | ConvertTo-Json -Compress

        $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).actualState

        ($result.value | ConvertTo-Json -Compress) |
            Should -BeExactly '["C:\\One","C:\\Two"]'
        $result.PSObject.Properties.Name | Should -Not -Contain 'delimiter'
        $result.PSObject.Properties.Name | Should -Not -Contain 'setAction'
    }

    It 'Sets entries using a custom delimiter' {
        $json = @{
            name      = $testName
            value     = @('one', 'two')
            delimiter = '::'
        } | ConvertTo-Json -Compress

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState

        ($result.value | ConvertTo-Json -Compress) | Should -BeExactly '["one","two"]'
        (Get-ItemPropertyValue -Path 'HKCU:\Environment' -Name $testName) |
            Should -BeExactly 'one::two'
    }

    It 'Appends entries and tests the projected state' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName -Value 'C:\Existing' -Type String
        $json = @{
            name      = $testName
            value     = @('C:\New')
            setAction = 'append'
        } | ConvertTo-Json -Compress

        $set = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        (($set | ConvertFrom-Json).afterState.value | ConvertTo-Json -Compress) |
            Should -BeExactly '["C:\\Existing","C:\\New"]'

        $test = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        ($test | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Removes a path variable when _exist is false' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testName -Value 'remove me' -Type String
        $json = @{ name = $testName; _exist = $false } | ConvertTo-Json -Compress

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        ($out | ConvertFrom-Json).afterState._exist | Should -BeFalse
    }
}
