# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Parameters tests' {
    It 'Input can be provided as <inputType>' -TestCases @(
        @{ inputType = 'string' }
        @{ inputType = 'file' }
    ) {
        param($inputType)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: string
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = 'hello' }} | ConvertTo-Json

        if ($inputType -eq 'file') {
            $file_path = "$TestDrive/test.parameters.json"
            Set-Content -Path $file_path -Value $params_json
            $out = $config_yaml | dsc config -f $file_path get -f - | ConvertFrom-Json
        }
        else {
            $out = $config_yaml | dsc config -p $params_json get -f - | ConvertFrom-Json
        }

        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'hello'
    }

    It 'Input is <type>' -TestCases @(
        @{ type = 'string'; value = 'hello' }
        @{ type = 'int'; value = 42}
        @{ type = 'bool'; value = $true}
        @{ type = 'array'; value = @('hello', 'world')}
    ) {
        param($type, $value)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: $type
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $out = $config_yaml | dsc config -p $params_json get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly $value
    }

    It 'Input is incorrect type <type>' -TestCases @(
        @{ type = 'string'; value = 42 }
        @{ type = 'int'; value = 'hello' }
        @{ type = 'bool'; value = 'hello' }
        @{ type = 'array'; value = 'hello' }
    ) {
        param($type, $value)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: $type
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $testError = & {$config_yaml | dsc config -p $params_json get -f - 2>&1}
        $testError | Should -match 'Parameter input failure:'
        $LASTEXITCODE | Should -Be 4
    }

    It 'Input length is wrong for <type> with value: <value>' -TestCases @(
        @{ type = 'string'; value = 'hi' }
        @{ type = 'string'; value = 'hello' }
        @{ type = 'array'; value = @('hello', 'there') }
        @{ type = 'array'; value = @('hello', 'there', 'bye', 'now') }
    ) {
        param($type, $value)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: $type
                minLength: 3
                maxLength: 3
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $testError = & {$config_yaml | dsc config -p $params_json get -f - 2>&1}
        $testError[0] | Should -match 'error'
        $LASTEXITCODE | Should -Be 4
    }

    It 'Input number value is out of range for <min> and <max>' -TestCases @(
        @{ value = 42; min = 43; max = 44 }
        @{ value = 42; min = 41; max = 41 }
        @{ value = 42; min = 43; max = 41 }
    ) {
        param($type, $value, $min, $max)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: int
                minValue: $min
                maxValue: $max
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $testError = & {$config_yaml | dsc config -p $params_json get -f - 2>&1}
        $testError[0] | Should -match 'error'
        $LASTEXITCODE | Should -Be 4
    }

    It 'Input is not in the allowed value list for <type>' -TestCases @(
        @{ type = 'string'; value = 'hello'; allowed = @('world', 'planet') }
        @{ type = 'int'; value = 42; allowed = @(43, 44) }
    ) {
        param($type, $value, $allowed)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: $type
                allowedValues: $($allowed | ConvertTo-Json -Compress)
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $testError = & {$config_yaml | dsc config -p $params_json get -f - 2>&1}
        $testError[0] | Should -match 'error'
        $LASTEXITCODE | Should -Be 4
    }

    It 'Length constraint is incorrectly applied to <type> with <constraint>' -TestCases @(
        @{ type = 'int'; value = 42; constraint = 'minLength' }
        @{ type = 'int'; value = 42; constraint = 'maxLength' }
        @{ type = 'bool'; value = $true; constraint = 'minLength' }
        @{ type = 'bool'; value = $true; constraint = 'maxLength' }
    ) {
        param($type, $value, $constraint)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: $type
                ${constraint}: 3
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $testError = & {$config_yaml | dsc config -p $params_json get -f - 2>&1}
        $testError[0] | Should -match 'error'
        $LASTEXITCODE | Should -Be 4
    }

    It 'Default value is used when not provided' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              paramString:
                type: string
                defaultValue: 'hello'
              paramInt:
                type: int
                defaultValue: 7
              paramBool:
                type: bool
                defaultValue: false
              paramArray:
                type: array
                defaultValue: ['hello', 'world']
            resources:
            - name: String
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''paramString'')]'
            - name: Int
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''paramInt'')]'
            - name: Bool
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''paramBool'')]'
            - name: Array
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '[parameters(''paramArray'')]'
"@

        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'hello'
        $out.results[1].result.actualState.output | Should -BeExactly 7
        $out.results[2].result.actualState.output | Should -BeExactly $false
        $out.results[3].result.actualState.output | Should -BeExactly @('hello', 'world')
    }

    It 'property value uses parameter value' {
      $os = 'Windows'
      if ($IsLinux) {
        $os = 'Linux'
      }
      elseif ($IsMacOS) {
        $os = 'macOS'
      }

      $params = @{
        parameters = @{
          osFamily = $os
        }
      } | ConvertTo-Json

      $config_yaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        parameters:
          osFamily:
            type: string
            defaultValue: Windows
            allowedValues:
              - Windows
              - Linux
              - macOS
        resources:
        - name: os
          type: Microsoft/OSInfo
          properties:
            family: '[parameters(''osFamily'')]'
'@

      $out = dsc config -p $params test -i $config_yaml | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results[0].result.actualState.family | Should -BeExactly $os
      $out.results[0].result.inDesiredState | Should -BeTrue
    }

    It 'secure types can be passed as objects to resources but redacted in output: <operation> <property>' -TestCases @(
        @{ operation = 'get'; property = 'actualState' }
        @{ operation = 'set'; property = 'beforeState' }
        @{ operation = 'set'; property = 'afterState' }
        @{ operation = 'test'; property = 'desiredState' }
        @{ operation = 'test'; property = 'actualState' }
        @{ operation = 'export'; property = $null }
    ) {
      param($operation, $property)

      $out = dsc -l trace config -f $PSScriptRoot/../examples/secure_parameters.parameters.yaml $operation -f $PSScriptRoot/../examples/secure_parameters.dsc.yaml 2> $TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      if ($operation -eq 'export') {
        $out.resources.Count | Should -Be 4
        $out.resources[0].properties.output | Should -BeExactly '<secureValue>'
        $out.resources[1].properties.output | Should -BeExactly '<secureValue>'
        $out.resources[2].properties.output[0] | Should -BeExactly '<secureValue>'
        $out.resources[2].properties.output[1] | Should -BeExactly '<secureValue>'
        $out.resources[3].properties.output | Should -BeExactly '<secureValue>'
      } else {
        $out.results.Count | Should -Be 4 -Because ($out | ConvertTo-Json -Dep 10 | Out-String)
        $out.results[0].result.$property.output | Should -BeExactly '<secureValue>' -Because ($out | ConvertTo-Json -Dep 10 | Out-String)
        $out.results[1].result.$property.output | Should -BeExactly '<secureValue>'
        $out.results[2].result.$property.output[0] | Should -BeExactly '<secureValue>'
        $out.results[2].result.$property.output[1] | Should -BeExactly '<secureValue>'
        $out.results[3].result.$property.output | Should -BeExactly '<secureValue>'
      }
    }

    It 'secure types can be passed as objects to resources: <operation> <property>' -TestCases @(
        # `set` beforeState is redacted in output, `test` desiredState is redacted in output so those test cases are not included here
        @{ operation = 'get'; property = 'actualState' }
        @{ operation = 'set'; property = 'afterState' }
        @{ operation = 'test'; property = 'actualState' }
        @{ operation = 'export'; property = $null }
    ) {
      param($operation, $property)

      $out = dsc config -f $PSScriptRoot/../examples/secure_parameters_shown.parameters.yaml $operation -f $PSScriptRoot/../examples/secure_parameters.dsc.yaml | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      if ($operation -eq 'export') {
        $out.resources.Count | Should -Be 4 -Because ($out | ConvertTo-Json -Dep 10 | Out-String)
        $out.resources[0].properties.output.secureString | Should -BeExactly 'mySecret'
        $out.resources[1].properties.output.secureString | Should -BeExactly 'mySecretProperty'
        $out.resources[2].properties.output[0].secureString | Should -BeExactly 'item1'
        $out.resources[2].properties.output[1].secureString | Should -BeExactly 'item2'
        $out.resources[3].properties.output.secureObject.secureString | Should -BeExactly 'item2'
      } else {
        $out.results.Count | Should -Be 4
        $out.results[0].result.$property.output.secureString | Should -BeExactly 'mySecret' -Because ($out | ConvertTo-Json -Dep 10 | Out-String)
        $out.results[1].result.$property.output.secureString | Should -BeExactly 'mySecretProperty'
        $out.results[2].result.$property.output[0].secureString | Should -BeExactly 'item1'
        $out.results[2].result.$property.output[1].secureString | Should -BeExactly 'item2'
        $out.results[3].result.$property.output.secureObject.secureString | Should -BeExactly 'item2'
      }
    }

    It 'parameter types are validated for <type>' -TestCases @(
      @{ type = 'array'; value = 'hello'}
      @{ type = 'bool'; value = 'hello'}
      @{ type = 'int'; value = @(1,2)}
      @{ type = 'object'; value = 1}
      @{ type = 'secureString'; value = 1}
      @{ type = 'secureObject'; value = 'hello'}
      @{ type = 'string'; value = 42 }
    ){
      param($type, $value)

      $config_yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        parameters:
          param:
            type: $type
        resources:
        - name: Echo
          type: Microsoft.DSC.Debug/Echo
          properties:
            output: '[parameters(''param'')]'
"@

      $params_json = @{ parameters = @{ param = $value }} | ConvertTo-Json
      $output = $config_yaml | dsc config -p $params_json get -f - 2>&1
      $LASTEXITCODE | Should -Be 4
      if ($type -eq 'secureString') {
        $type = 'string'
      }
      elseif ($type -eq 'secureObject') {
        $type = 'object'
      }

      $output | Should -Match "Parameter input failure:.*?$type"
    }

    It 'Parameters can be read from STDIN' {
      $params = @{
        parameters = @{
          osFamily = 'Windows'
        }
      } | ConvertTo-Json -Compress

      $out = $params | dsc config -f - test -f "$PSScriptRoot/../examples/osinfo_parameters.dsc.yaml" 2> $TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Path $TestDrive/error.log | Out-String)
      $out.results[0].result.desiredState.family | Should -BeExactly 'Windows'
      $out.results[0].result.inDesiredState | Should -Be $IsWindows
    }

    It 'Parameters and input cannot both be from STDIN' {
      $params = @{
        parameters = @{
          osFamily = 'Windows'
        }
      } | ConvertTo-Json -Compress

      $out = $params | dsc config -f - test -f - 2> $TestDrive/error.log
      $LASTEXITCODE | Should -Be 4
      $out | Should -BeNullOrEmpty
      $errorMessage = Get-Content -Path $TestDrive/error.log -Raw
      $errorMessage | Should -BeLike "*ERROR*Cannot read from STDIN for both parameters and input*"
    }

    It 'Invalid parameters read from STDIN result in error' {
      $params = @{
        osFamily = 'Windows'
      } | ConvertTo-Json -Compress

      $out = $params | dsc config -f - test -f "$PSScriptRoot/../examples/osinfo_parameters.dsc.yaml" 2> $TestDrive/error.log
      $LASTEXITCODE | Should -Be 4
      $out | Should -BeNullOrEmpty
      $errorMessage = Get-Content -Path $TestDrive/error.log -Raw
      $errorMessage | Should -BeLike "*ERROR*Parameter input failure: JSON: missing field ````parameters````*"
    }

    It 'Parameters can reference other parameters in defaultValue: simple nested' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              basePrefix:
                type: string
                defaultValue: base
              computedPrefix:
                type: string
                defaultValue: "[concat(parameters('basePrefix'), '-computed')]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('computedPrefix')]"
"@

        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'base-computed'
    }

    It 'Parameters can reference other parameters in defaultValue: multi-level nested' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              environment:
                type: string
                defaultValue: dev
              appName:
                type: string
                defaultValue: "[concat(parameters('environment'), '-myapp')]"
              instanceName:
                type: string
                defaultValue: "[concat(parameters('appName'), '-001')]"
              fullInstanceName:
                type: string
                defaultValue: "[concat('Instance: ', parameters('instanceName'))]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('fullInstanceName')]"
"@

        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'Instance: dev-myapp-001'
    }

    It 'Parameters with circular dependencies are detected' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              paramA:
                type: string
                defaultValue: "[parameters('paramB')]"
              paramB:
                type: string
                defaultValue: "[parameters('paramA')]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('paramA')]"
"@

        $testError = & {$config_yaml | dsc config get -f - 2>&1}
        $LASTEXITCODE | Should -Be 4
        $testError | Should -Match 'Circular dependency or unresolvable parameter references detected in parameters'
    }

    It 'Parameters with complex circular dependencies are detected' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              paramA:
                type: string
                defaultValue: "[parameters('paramB')]"
              paramB:
                type: string
                defaultValue: "[parameters('paramC')]"
              paramC:
                type: string
                defaultValue: "[parameters('paramA')]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('paramA')]"
"@

        $testError = & {$config_yaml | dsc config get -f - 2>&1}
        $LASTEXITCODE | Should -Be 4
        $testError | Should -Match 'Circular dependency or unresolvable parameter references detected in parameters'
    }

    It 'Parameters with nested references can be overridden by input' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              basePrefix:
                type: string
                defaultValue: base
              computedPrefix:
                type: string
                defaultValue: "[concat(parameters('basePrefix'), '-computed')]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('computedPrefix')]"
"@
        $params_json = @{ parameters = @{ basePrefix = 'override' }} | ConvertTo-Json

        $out = $config_yaml | dsc config -p $params_json get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'override-computed'
    }

    It 'Parameters nested references work with different data types: <type>' -TestCases @(
        @{ type = 'string'; baseValue = 'test'; expectedOutput = 'prefix-test-suffix' }
        @{ type = 'int'; baseValue = 42; expectedOutput = 'value-42' }
    ) {
        param($type, $baseValue, $expectedOutput)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              baseParam:
                type: $type
                defaultValue: $baseValue
              computedParam:
                type: string
                defaultValue: "[concat('prefix-', string(parameters('baseParam')), '-suffix')]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('computedParam')]"
"@

        if ($type -eq 'string') {
            $expectedOutput = 'prefix-test-suffix'
        } else {
            $expectedOutput = 'prefix-42-suffix'
        }

        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly $expectedOutput
    }

    It 'Parameters with unresolvable references produce error' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              computedParam:
                type: string
                defaultValue: "[parameters('nonExistentParam')]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('computedParam')]"
"@

        $testError = & {$config_yaml | dsc config get -f - 2>&1}
        $LASTEXITCODE | Should -Be 4
        $testError | Should -Match 'Circular dependency or unresolvable parameter references detected in parameters'
    }

    It 'Inline parameters and parameters file can be used together' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              message:
                type: string
              environment:
                type: string
              port:
                type: int
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[concat('Message: ', parameters('message'), ', Environment: ', parameters('environment'), ', Port: ', string(parameters('port')))]"
"@
        $paramsFile = @{
            parameters = @{
                message = 'Hello from file'
                environment = 'production'
                port = 8080
            }
        } | ConvertTo-Json

        $inlineParams = @{
            parameters = @{
                message = 'Hello from inline'
                port = 3000
            }
        } | ConvertTo-Json

        $file_path = "$TestDrive/test.parameters.json"
        Set-Content -Path $file_path -Value $paramsFile

        $out = dsc config --parameters-file $file_path --parameters $inlineParams get -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'Message: Hello from inline, Environment: production, Port: 3000'
    }

    It 'Inline parameters take precedence over parameters file' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              value1:
                type: string
              value2:
                type: string
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[concat(parameters('value1'), '-', parameters('value2'))]"
"@
        $paramsFile = @{
            parameters = @{
                value1 = 'file1'
                value2 = 'file2'
            }
        } | ConvertTo-Json

        $inlineParams = @{
            parameters = @{
                value1 = 'inline1'
            }
        } | ConvertTo-Json

        $file_path = "$TestDrive/test.parameters.json"
        Set-Content -Path $file_path -Value $paramsFile

        $out = dsc config --parameters-file $file_path --parameters $inlineParams get -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'inline1-file2'
    }

    It 'Parameters file from stdin can be merged with inline parameters' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: string
              param2:
                type: string
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[concat(parameters('param1'), '-', parameters('param2'))]"
"@
        $paramsFile = @{
            parameters = @{
                param1 = 'fromStdin'
                param2 = 'alsoFromStdin'
            }
        } | ConvertTo-Json

        $inlineParams = @{
            parameters = @{
                param2 = 'fromInline'
            }
        } | ConvertTo-Json

        $out = $paramsFile | dsc config --parameters-file - --parameters $inlineParams get -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'fromStdin-fromInline'
    }

    It 'Parameters file in YAML format can be merged with inline JSON parameters' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              name:
                type: string
              age:
                type: int
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[concat(parameters('name'), ' is ', string(parameters('age')))]"
"@
        $paramsFileYaml = @"
parameters:
  name: John
  age: 30
"@
        $inlineParams = @{
            parameters = @{
                age = 25
            }
        } | ConvertTo-Json

        $file_path = "$TestDrive/test.parameters.yaml"
        Set-Content -Path $file_path -Value $paramsFileYaml

        $out = dsc config --parameters-file $file_path --parameters $inlineParams get -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'John is 25'
    }

    It 'Merged parameters work with default values' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: string
                defaultValue: default1
              param2:
                type: string
                defaultValue: default2
              param3:
                type: string
                defaultValue: default3
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[concat(parameters('param1'), '-', parameters('param2'), '-', parameters('param3'))]"
"@
        $paramsFile = @{
            parameters = @{
                param2 = 'fromFile'
            }
        } | ConvertTo-Json

        $inlineParams = @{
            parameters = @{
                param3 = 'fromInline'
            }
        } | ConvertTo-Json

        $file_path = "$TestDrive/test.parameters.json"
        Set-Content -Path $file_path -Value $paramsFile

        $out = dsc config --parameters-file $file_path --parameters $inlineParams get -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'default1-fromFile-fromInline'
    }

    It 'Merged parameters support all data types: <type>' -TestCases @(
        @{ type = 'string'; fileValue = 'file'; inlineValue = 'inline' }
        @{ type = 'int'; fileValue = 100; inlineValue = 200 }
        @{ type = 'bool'; fileValue = $false; inlineValue = $true }
        @{ type = 'array'; fileValue = @('a', 'b'); inlineValue = @('x', 'y', 'z') }
        @{ type = 'object'; fileValue = @{key1='value1'}; inlineValue = @{key2='value2'} }
    ) {
        param($type, $fileValue, $inlineValue)

        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              fileParam:
                type: $type
              inlineParam:
                type: $type
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[concat(string(parameters('fileParam')), '-', string(parameters('inlineParam')))]"
"@
        $paramsFile = @{
            parameters = @{
                fileParam = $fileValue
                inlineParam = $fileValue
            }
        } | ConvertTo-Json

        $inlineParams = @{
            parameters = @{
                inlineParam = $inlineValue
            }
        } | ConvertTo-Json

        $file_path = "$TestDrive/test.parameters.json"
        Set-Content -Path $file_path -Value $paramsFile

        $out = dsc config --parameters-file $file_path --parameters $inlineParams get -i $config_yaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($type -eq 'bool') {
            $expectedOutput = "{0}-{1}" -f $fileValue.ToString().ToLower(), $inlineValue.ToString().ToLower()
        }
        elseif ($type -eq 'array') {
            $fileJson = $fileValue | ConvertTo-Json -Compress
            $inlineJson = $inlineValue | ConvertTo-Json -Compress
            $expectedOutput = "{0}-{1}" -f $fileJson, $inlineJson
        }
        elseif ($type -eq 'object') {
            $fileJson = $fileValue | ConvertTo-Json -Compress
            $inlineJson = $inlineValue | ConvertTo-Json -Compress
            $expectedOutput = "{0}-{1}" -f $fileJson, $inlineJson
        }
        else {
            $expectedOutput = "{0}-{1}" -f $fileValue, $inlineValue
        } 
        $out.results[0].result.actualState.output | Should -BeExactly $expectedOutput
    }

    It 'Invalid inline parameters format returns error when merging' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: string
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('param1')]"
"@
        $paramsFile = @{
            parameters = @{
                param1 = 'valid'
            }
        } | ConvertTo-Json

        $file_path = "$TestDrive/test.parameters.json"
        Set-Content -Path $file_path -Value $paramsFile

        $invalidInlineParams = 'not valid json'

        $out = & {$config_yaml | dsc config --parameters-file $file_path --parameters $invalidInlineParams get -f - 2>&1}
        $LASTEXITCODE | Should -Be 4
        $out | Should -Match 'Failed to merge parameters'
    }

    It 'Invalid parameters file format returns error when merging' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              param1:
                type: string
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('param1')]"
"@
        $inlineParams = @{
            parameters = @{
                param1 = 'valid'
            }
        } | ConvertTo-Json

        $file_path = "$TestDrive/test.parameters.json"
        Set-Content -Path $file_path -Value 'not valid json'

        $out = & {$config_yaml | dsc config --parameters-file $file_path --parameters $inlineParams get -f - 2>&1}
        $LASTEXITCODE | Should -Be 4
        $out | Should -Match 'Failed to merge parameters'
    }
}
