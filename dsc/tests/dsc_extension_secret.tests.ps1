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
                showSecrets: true
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
                showSecrets: true
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
                showSecrets: true
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
                showSecrets: true
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
                showSecrets: true
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
                showSecrets: true
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
                showSecrets: true
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
                showSecrets: true
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
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/error.log)
      $out.results.Count | Should -Be 1
      $out.results[0].result.actualState.Output | Should -BeExactly 'Hello' -Because (Get-Content -Raw -Path $TestDrive/error.log)
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
              showSecrets: true
'@
      $out = dsc -l trace config get -i $configYaml 2> $TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results.Count | Should -Be 1
      $out.results[0].result.actualState.Output | Should -BeExactly 'Hello'
    }

    It 'Deprecated extension shows message' {
      try {
        $dscHome = Split-Path (Get-Command dsc).Source -Parent
        $env:DSC_RESTRICTED_PATH = (Join-Path -Path $dscHome -ChildPath 'deprecated') + [System.IO.Path]::PathSeparator + $dscHome

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
      } finally {
        $env:DSC_RESTRICTED_PATH = $null
      }
    }

    It 'Secret extension manifest <reason> is not loaded' -TestCases @(
        @{ reason = 'without args'; secret = '{ "executable": "dsctest" }'; expectedError = 'missing field *args*' }
        @{ reason = 'without a name argument'; secret = '{ "executable": "dsctest", "args": ["no-op"] }'; expectedError = "The 'secret' command doesn't define the secret name input argument" }
        @{ reason = 'with multiple name arguments'; secret = '{ "executable": "dsctest", "args": ["no-op", { "nameArg": "--name" }, { "nameArg": "--secret" }] }'; expectedError = "The 'secret' command defines the secret name input argument 2 times" }
        @{ reason = 'with multiple vault arguments'; secret = '{ "executable": "dsctest", "args": ["no-op", { "nameArg": "--name" }, { "vaultArg": "--vault" }, { "vaultArg": "--store" }] }'; expectedError = "The 'secret' command defines the vault input argument 2 times" }
    ) {
        param($secret, $expectedError)

        $manifest = @"
{
    "`$schema": "https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json",
        "type": "Test/SecretInvalid",
        "version": "0.1.0",
        "description": "Invalid secret extension for testing.",
        "secret": $secret
}
"@

        try {
            $env:DSC_RESTRICTED_PATH = $TestDrive
            Set-Content -Path "$TestDrive/secretInvalid.dsc.extension.json" -Value $manifest
            $out = dsc -l info extension list 2> $TestDrive/error.log | ConvertFrom-Json
            $errorLog = Get-Content -Raw -Path $TestDrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because $errorLog
            @($out).type | Should -Not -Contain 'Test/SecretInvalid'
            $errorLog | Should -BeLike "*INFO Failed to load manifest: *$expectedError*" -Because $errorLog
        } finally {
            $env:DSC_RESTRICTED_PATH = $null
        }
    }

    It 'Secret extension manifest with a name argument and a vault argument is loaded' {
        $manifest = @'
{
    "$schema": "https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json",
    "type": "Test/SecretValid",
    "version": "0.1.0",
    "description": "Valid secret extension for testing.",
    "secret": {
        "executable": "dsctest",
        "args": [
            "no-op",
            { "vaultArg": "--vault" },
            { "nameArg": "--name" }
        ]
    }
}
'@

        try {
            $env:DSC_RESTRICTED_PATH = $TestDrive
            Set-Content -Path "$TestDrive/secretValid.dsc.extension.json" -Value $manifest
            $out = dsc extension list 2> $TestDrive/error.log | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw -Path $TestDrive/error.log)
            @($out).Count | Should -Be 1
            $out.type | Should -BeExactly 'Test/SecretValid'
            $out.capabilities | Should -BeExactly @('secret')
        } finally {
            $env:DSC_RESTRICTED_PATH = $null
        }
    }
}
