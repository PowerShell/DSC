# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'metadata tests' {
    It 'metadata not provided if not declared in resource schema' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Microsoft.DSC.Debug/Echo
            metadata:
              ignoreKey: true
            properties:
              output: hello world
'@
        $out = dsc -l info config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        (Get-Content $TestDrive/error.log -Raw) | Should -BeLike "*INFO Will not add '_metadata' to properties because resource schema does not support it*" -Because (Get-Content $TestDrive/error.log -Raw)
        $out.results.result.actualState.output | Should -BeExactly 'hello world'
    }

    It 'resource can provide high-level metadata for <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            metadata:
              hello: world
              myNumber: 42
            properties:
'@

        $out = dsc config $operation -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.hello | Should -BeExactly 'world'
        $out.results[0].metadata.myNumber | Should -Be 42
    }

    It 'resource can provide high-level metadata for export' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            metadata:
              hello: There
              myNumber: 16
            properties:
'@
        $out = dsc config export -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 3
        $out.resources[0].metadata.hello | Should -BeExactly 'There'
        $out.resources[0].metadata.myNumber | Should -Be 16
        $out.resources[0].name | Should -BeExactly 'Metadata example 1'
        $out.resources[1].metadata.hello | Should -BeExactly 'There'
        $out.resources[1].metadata.myNumber | Should -Be 16
        $out.resources[1].name | Should -BeExactly 'Metadata example 2'
        $out.resources[2].metadata.hello | Should -BeExactly 'There'
        $out.resources[2].metadata.myNumber | Should -Be 16
        $out.resources[2].name | Should -BeExactly 'Metadata example 3'
    }

    It 'resource can provide metadata for <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                hello: world
                myNumber: 42
'@

        $out = dsc config $operation -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.hello | Should -BeExactly 'world'
        $out.results[0].metadata.myNumber | Should -Be 42
    }

    It 'resource can provide metadata for export' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                hello: There
                myNumber: 16
'@
        $out = dsc config export -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.resources.count | Should -Be 3
        $out.resources[0].metadata.hello | Should -BeExactly 'There'
        $out.resources[0].metadata.myNumber | Should -Be 16
        $out.resources[0].name | Should -BeExactly 'Metadata example 1'
        $out.resources[1].metadata.hello | Should -BeExactly 'There'
        $out.resources[1].metadata.myNumber | Should -Be 16
        $out.resources[1].name | Should -BeExactly 'Metadata example 2'
        $out.resources[2].metadata.hello | Should -BeExactly 'There'
        $out.resources[2].metadata.myNumber | Should -Be 16
        $out.resources[2].name | Should -BeExactly 'Metadata example 3'
    }

    It 'resource returning Microsoft.DSC metadata is ignored' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/Metadata
            properties:
              _metadata:
                Microsoft.DSC:
                  hello: world
                validOne: true
'@
        $out = dsc config get -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results.count | Should -Be 1
        $out.results[0].metadata.validOne | Should -BeTrue
        $out.results[0].metadata.Microsoft.DSC | Should -BeNullOrEmpty
        (Get-Content $TestDrive/error.log) | Should -BeLike "*WARN*Resource returned '_metadata' property 'Microsoft.DSC' which is ignored*"
    }

    It '_refreshEnv refreshes the environment variables for subsequent resources' {
        if ($IsWindows) {
            Remove-Item -Path "HKCU:\Environment\myTestVariable" -ErrorAction Ignore
        }

        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: create variable
            type: Test/RefreshEnv
            properties:
              name: myTestVariable
              value: myTestValue
          - name: return variable
            type: Microsoft.DSC.Transitional/PowerShellScript
            properties:
              SetScript: |
                if ($IsWindows) {
                  $env:myTestVariable
                }
'@
        try {
          $out = dsc -l trace config set -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
          $errorLogContent = Get-Content $TestDrive/error.log -Raw
          $LASTEXITCODE | Should -Be 0 -Because $errorLogContent
          if ($IsWindows) {
              $errorLogContent | Should -BeLike "*Resource returned '_refreshEnv' which indicates environment variable refresh is needed*" -Because $errorLogContent
              $out.results[1].result.afterState.output | Should -BeExactly 'myTestValue' -Because ($out | ConvertTo-Json -Depth 10)
          } else {
              $errorLogContent | Should -BeLike "*INFO*Resource returned '_refreshEnv' which is ignored on non-Windows platforms*" -Because $errorLogContent
          }
        } finally {
            if ($IsWindows) {
                Remove-Item -Path "HKCU:\Environment\myTestVariable" -ErrorAction Ignore
            }
        }
    }

    It '_refreshEnv handles PATH correctly' -Skip:(!$IsWindows) {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: add to path
            type: Test/RefreshEnv
            properties:
              name: PATH
              value: C:\MyTestPath
          - name: return path
            type: Microsoft.DSC.Transitional/PowerShellScript
            properties:
              SetScript: |
                $env:PATH
'@
        $oldUserPath = [System.Environment]::GetEnvironmentVariable('PATH', [System.EnvironmentVariableTarget]::User)
        try {
          $out = dsc -l trace config set -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
          $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
          $out.results[1].result.afterState.output.Split(';') | Should -Contain 'C:\MyTestPath'
        } finally {
            [System.Environment]::SetEnvironmentVariable('PATH', $oldUserPath, [System.EnvironmentVariableTarget]::User)
        }
    }

    It '_refreshEnv does not trigger for <operation>' -Skip:(!$IsWindows) -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
          - name: test
            type: Test/RefreshEnv
            properties:
              name: myTestVariable
              value: myTestValue
          - name: return variable
            type: Microsoft.DSC.Transitional/PowerShellScript
            properties:
              SetScript: |
                if ($IsWindows) {
                  $env:myTestVariable
                }
'@
        $out = dsc -l trace config $operation -i $configYaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
        (Get-Content $TestDrive/error.log -Raw) | Should -BeLike "*Resource returned '_refreshEnv' which indicates environment variable refresh is needed but current operation is '$operation' which is not 'set', so ignoring*" -Because (Get-Content $TestDrive/error.log -Raw)
        $out.results[0].result.afterState.output | Should -Not -Be 'myTestValue'
    }
}
