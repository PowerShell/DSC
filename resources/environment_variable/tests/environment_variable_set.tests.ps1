# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentVariableList set operation' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentVariableList'
        $namePrefix = "DSC_Environment_Set_$([guid]::NewGuid().ToString('N'))"
        $testNames = @(
            "${namePrefix}_Scalar"
            "${namePrefix}_Path"
            "${namePrefix}_First"
            "${namePrefix}_Second"
        )
    }

    AfterEach {
        foreach ($name in $testNames) {
            Remove-ItemProperty -Path 'HKCU:\Environment' -Name $name -ErrorAction Ignore
        }
    }

    It 'Sets a scalar value with CurrentUser and _exist defaults' {
        $json = @{
            environmentVariables = @(
                @{
                    name  = $testNames[0]
                    value = 'DSC scalar value'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState.environmentVariables[0]

        $result.scope | Should -BeExactly 'currentUser'
        $result.value | Should -BeExactly 'DSC scalar value'
        $result._exist | Should -BeTrue
        [Environment]::GetEnvironmentVariable(
            $testNames[0],
            [EnvironmentVariableTarget]::User) | Should -BeExactly 'DSC scalar value'
    }

    It 'Clobbers a path value by default and removes duplicate entries case-insensitively' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testNames[1] -Value 'C:\Old' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name      = $testNames[1]
                    pathValue = @('C:\One', 'c:\one', 'C:\Two')
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState.environmentVariables[0]

        ($result.pathValue | ConvertTo-Json -Compress) |
            Should -BeExactly '["C:\\One","C:\\Two"]'
        [Environment]::GetEnvironmentVariable(
            $testNames[1],
            [EnvironmentVariableTarget]::User) | Should -BeExactly 'C:\One;C:\Two'
    }

    It 'Prepends path entries and moves an existing duplicate to the front' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testNames[1] `
            -Value 'C:\Existing;C:\Shared' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name       = $testNames[1]
                    pathValue  = @('c:\shared', 'C:\New')
                    pathAction = 'prepend'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState.environmentVariables[0]

        ($result.pathValue | ConvertTo-Json -Compress) |
            Should -BeExactly '["c:\\shared","C:\\New","C:\\Existing"]'
    }

    It 'Appends path entries and moves an existing duplicate to the end' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testNames[1] `
            -Value 'C:\Shared;C:\Existing' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name       = $testNames[1]
                    pathValue  = @('c:\shared', 'C:\New')
                    pathAction = 'append'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState.environmentVariables[0]

        ($result.pathValue | ConvertTo-Json -Compress) |
            Should -BeExactly '["C:\\Existing","c:\\shared","C:\\New"]'
    }

    It 'Removes a variable when _exist is false' {
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $testNames[0] -Value 'remove me' -Type String
        $json = @{
            environmentVariables = @(
                @{
                    name   = $testNames[0]
                    _exist = $false
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState.environmentVariables[0]

        $result._exist | Should -BeFalse
        [Environment]::GetEnvironmentVariable(
            $testNames[0],
            [EnvironmentVariableTarget]::User) | Should -BeNullOrEmpty
    }

    It 'Sets multiple variables in one request' {
        $json = @{
            environmentVariables = @(
                @{
                    name  = $testNames[2]
                    value = 'first'
                }
                @{
                    name  = $testNames[3]
                    value = 'second'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $result = ($out | ConvertFrom-Json).afterState.environmentVariables

        $result.Count | Should -Be 2
        $result[0].value | Should -BeExactly 'first'
        $result[1].value | Should -BeExactly 'second'
    }

    It 'Rejects value and pathValue together' {
        $json = @{
            environmentVariables = @(
                @{
                    name      = $testNames[0]
                    value     = 'value'
                    pathValue = @('C:\Path')
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>&1
        $LASTEXITCODE | Should -Not -Be 0
        $out | Should -Match 'value.*pathValue'
    }

    It 'Returns an actionable elevation error for AllUsers' -Skip:$isAdmin {
        $machineName = "${namePrefix}_Machine"
        $json = @{
            environmentVariables = @(
                @{
                    scope = 'allUsers'
                    name  = $machineName
                    value = 'requires elevation'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource set -r $resourceType -f - 2>&1

        $LASTEXITCODE | Should -Not -Be 0
        $out | Should -Match 'elevation'
        [Environment]::GetEnvironmentVariable(
            $machineName,
            [EnvironmentVariableTarget]::Machine) | Should -BeNullOrEmpty
    }
}
