# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Microsoft.Windows/EnvironmentVariableList export operation' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/EnvironmentVariableList'
        $namePrefix = "DSC_Environment_Export_$([guid]::NewGuid().ToString('N'))"
        $scalarName = "${namePrefix}_Scalar"
        $pathName = "${namePrefix}_Path"
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $scalarName -Value 'scalar value' -Type String
        Set-ItemProperty -Path 'HKCU:\Environment' -Name $pathName -Value 'C:\One;C:\Two' -Type String
    }

    AfterAll {
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name $scalarName -ErrorAction Ignore
        Remove-ItemProperty -Path 'HKCU:\Environment' -Name $pathName -ErrorAction Ignore
    }

    It 'Exports all variables as scalar items by default' {
        $out = dsc resource export -r $resourceType 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $items = $out.resources[0].properties.environmentVariables
        $item = $items | Where-Object -Property name -EQ $pathName

        $item.value | Should -BeExactly 'C:\One;C:\Two'
        $item.scope | Should -BeExactly 'currentUser'
    }

    It 'Supports a case-insensitive asterisk wildcard name filter' {
        $json = @{
            environmentVariables = @(
                @{ name = "$($namePrefix.ToLower())*" }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource export -r $resourceType -f - 2>$testdrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $items = $out.resources[0].properties.environmentVariables

        $items.Count | Should -Be 2
        $items.name | Should -Contain $scalarName
        $items.name | Should -Contain $pathName
    }

    It 'Uses path representation for a path-shaped filter' {
        $json = @{
            environmentVariables = @(
                @{
                    name      = $pathName
                    value     = @()
                    delimiter = ';'
                }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource export -r $resourceType -f - 2>$testdrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $item = $out.resources[0].properties.environmentVariables[0]

        ($item.value | ConvertTo-Json -Compress) |
            Should -BeExactly '["C:\\One","C:\\Two"]'
    }

    It 'Applies AND within filters and OR across filters' {
        $json = @{
            environmentVariables = @(
                @{ scope = 'allUsers'; name = $scalarName }
                @{ scope = 'currentUser'; name = $pathName }
            )
        } | ConvertTo-Json -Compress -Depth 5

        $out = $json | dsc resource export -r $resourceType -f - 2>$testdrive/error.log |
            ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
        $items = $out.resources[0].properties.environmentVariables

        @($items).Count | Should -Be 1
        $items[0].name | Should -BeExactly $pathName
    }
}
