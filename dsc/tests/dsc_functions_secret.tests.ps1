# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for the secret() function and extensions' {
    BeforeAll {
        $oldPath = $env:PATH
        $toolPath = Resolve-Path -Path "$PSScriptRoot/../../extensions/test/secret"
        $env:PATH = "$toolPath" + [System.IO.Path]::PathSeparator + $oldPath
    }

    AfterAll {
        $env:PATH = $oldPath
    }

    It 'Call secret() function with just a name' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('MySecret')]"
'@
        $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/error.log)
        $out.results.Count | Should -Be 1
        $out.results[0].result.actualState.Output | Should -BeExactly 'Hello'
    }

    It 'Call secret() function with a name and vault' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('DifferentSecret', 'VaultA')]"
'@
        $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/error.log)
        $out.results.Count | Should -Be 1
        $out.results[0].result.actualState.Output | Should -BeExactly 'Hello2'
    }

    It 'Call secret() function with a name that does not exist' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('NonExistentSecret')]"
'@
        dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2
        $errorMessage = Get-Content -Raw -Path $TestDrive/error.log
        $errorMessage | Should -Match "Secret 'NonExistentSecret' not found"
    }

    It 'Call secret() function with a vault that does not exist' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('MySecret', 'NonExistentVault')]"
'@
        dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2
        $errorMessage = Get-Content -Raw -Path $TestDrive/error.log
        $errorMessage | Should -Match "Secret 'MySecret' not found"
    }

    It 'Call secret() function with a duplicate secret' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('DuplicateSecret')]"
'@
        dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2
        $errorMessage = Get-Content -Raw -Path $TestDrive/error.log
        $errorMessage | Should -Match "Multiple secrets with the same name 'DuplicateSecret' was returned, try specifying a vault"
    }

    It 'Call secret() function with secret and vault to disambiguate' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('DuplicateSecret', 'Vault1')]"
'@
        $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/error.log)
        $out.results.Count | Should -Be 1
        $out.results[0].result.actualState.Output | Should -BeExactly 'World'
    }
}
