# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentVariableList test operation' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentVariableList'
        $namePrefix = "DSC_Environment_Test_$([guid]::NewGuid().ToString('N'))"
        $scalarName = "${namePrefix}_Scalar"
        $pathName = "${namePrefix}_Path"
    }

    AfterEach {
        foreach ($name in @($scalarName, $pathName)) {
            Remove-ItemProperty -Path 'HKCU:\Environment' -Name $name -ErrorAction Ignore
        }
    }

    It 'Reports a matching scalar value in desired state' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $scalarName -Value 'expected' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name  = $scalarName
                    value = 'expected'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = $out | ConvertFrom-Json

        $result.inDesiredState | Should -BeTrue
        $result.actualState.environmentVariables[0].scope | Should -BeExactly 'currentUser'
    }

    It 'Reports a different scalar value outside desired state' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $scalarName -Value 'actual' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name  = $scalarName
                    value = 'expected'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeFalse
    }

    It 'Reports prepend in desired state after the requested entries are at the front' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $pathName `
            -Value 'C:\New;C:\Existing' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name       = $pathName
                    pathValue  = @('c:\new')
                    pathAction = 'prepend'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Reports prepend outside desired state before the requested entries are at the front' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $pathName -Value 'C:\Existing' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name       = $pathName
                    pathValue  = @('C:\New')
                    pathAction = 'prepend'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeFalse
    }

    It 'Reports append in desired state when the requested entries are at the end' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $pathName `
            -Value 'C:\Existing;C:\New' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name       = $pathName
                    pathValue  = @('C:\New')
                    pathAction = 'append'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }

    It 'Reports clobber outside desired state when extra entries exist' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $pathName `
            -Value 'C:\Expected;C:\Extra' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name       = $pathName
                    pathValue  = @('C:\Expected')
                    pathAction = 'clobber'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeFalse
    }

    It 'Reports a missing variable in desired state when _exist is false' {
        $json = @{
            environmentVariables = @(
                @{
                    name   = $scalarName
                    _exist = $false
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource test -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)

        ($out | ConvertFrom-Json).inDesiredState | Should -BeTrue
    }
}
