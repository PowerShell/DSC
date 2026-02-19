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

    It 'Just a secret name' {
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

    It 'Name and vault' {
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

    It 'Name that does not exist' {
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

    It 'Vault that does not exist' {
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

    It 'Duplicate secret' {
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
        $errorMessage | Should -Match "Multiple secrets with the same name 'DuplicateSecret' and different values was returned, try specifying a vault"
    }

    It 'Secret and vault to disambiguate' {
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

    It 'Same secret name and value in different extensions' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('DuplicateSame')]"
'@
        $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.Count | Should -Be 1
        $out.results[0].result.actualState.Output | Should -BeExactly 'SameSecret'
    }

    It 'Secret with multiple lines' {
        $configYaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[secret('MultiLine')]"
'@
        dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 2
        $errorMessage = Get-Content -Raw -Path $TestDrive/error.log
        $errorMessage | Should -Match "Extension 'Test/Secret2' returned multiple lines which is not supported for secrets"
    }

    It 'Allows to pass in secret() through parameters' {
      $configYaml = @'
          $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
          parameters:
            myString:
              type: secureString
              defaultValue: "[secret('MySecret')]"
          resources:
          - name: Database Connection
            type: Microsoft.DSC.Debug/Echo
            properties:
              output: "[parameters('myString')]"
              showSecrets: true
'@
      $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results.Count | Should -Be 1
      $out.results[0].result.actualState.Output.secureString | Should -BeExactly 'Hello'
    }

    It 'Allows to pass in secret() through variables' {
      $configYaml = @'
          $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
          variables:
            myString: "[secret('MySecret')]"
          resources:
          - name: Database Connection
            type: Microsoft.DSC.Debug/Echo
            properties:
              output: "[variables('myString')]"
'@
      $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results.Count | Should -Be 1
      $out.results[0].result.actualState.Output | Should -BeExactly 'Hello'
    }

    It 'Deprecated extension shows message' {
      $configYaml = @'
          $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
          variables:
            myString: "[secret('nonExisting')]"
          resources:
          - name: Database Connection
            type: Microsoft.DSC.Debug/Echo
            properties:
              output: "[variables('myString')]"
'@
      dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 4
      (Get-Content -Raw -Path "$TestDrive/error.log") | Should -Match "Extension 'Test/ExtensionDeprecated' is deprecated: This extension is deprecated" -Because (Get-Content -Raw -Path "$TestDrive/error.log")
    }
}
