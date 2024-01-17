# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Parameters tests' {
    It 'Input can be provided as <inputType>' -TestCases @(
        @{ inputType = 'string' }
        @{ inputType = 'file' }
    ) {
        param($inputType)

        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: string
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = 'hello' }} | ConvertTo-Json

        if ($inputType -eq 'file') {
            $file_path = "$TestDrive/test.parameters.json"
            Set-Content -Path $file_path -Value $params_json
            $out = $config_yaml | dsc config -f $file_path get | ConvertFrom-Json
        }
        else {
            $out = $config_yaml | dsc config -p $params_json get | ConvertFrom-Json
        }

        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.text | Should -BeExactly 'hello'
    }

    It 'Input is <type>' -TestCases @(
        @{ type = 'string'; value = 'hello'; expected = 'hello' }
        @{ type = 'int'; value = 42; expected = 42 }
        @{ type = 'bool'; value = $true; expected = $true }
        @{ type = 'array'; value = @('hello', 'world'); expected = '["hello","world"]' }
    ) {
        param($type, $value, $expected)

        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: $type
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $out = $config_yaml | dsc config -p $params_json get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.text | Should -BeExactly $expected
    }

    It 'Input is incorrect type <type>' -TestCases @(
        @{ type = 'string'; value = 42 }
        @{ type = 'int'; value = 'hello' }
        @{ type = 'bool'; value = 'hello' }
        @{ type = 'array'; value = 'hello' }
    ) {
        param($type, $value)

        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: $type
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $null = $config_yaml | dsc config -p $params_json get
        $LASTEXITCODE | Should -Be 4
    }

    It 'Input length is wrong for <type>' -TestCases @(
        @{ type = 'string'; value = 'hi' }
        @{ type = 'string'; value = 'hello' }
        @{ type = 'array'; value = @('hello', 'there') }
        @{ type = 'array'; value = @('hello', 'there', 'bye', 'now') }
    ) {
        param($type, $value)

        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: $type
                minLength: 3
                maxLength: 3
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $null = $config_yaml | dsc config -p $params_json get
        $LASTEXITCODE | Should -Be 4
    }

    It 'Input number value is out of range for <min> and <max>' -TestCases @(
        @{ value = 42; min = 43; max = 44 }
        @{ value = 42; min = 41; max = 41 }
        @{ value = 42; min = 43; max = 41 }
    ) {
        param($type, $value, $min, $max)

        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: int
                minValue: $min
                maxValue: $max
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $null = $config_yaml | dsc config -p $params_json get
        $LASTEXITCODE | Should -Be 4
    }

    It 'Input is not in the allowed value list for <type>' -TestCases @(
        @{ type = 'string'; value = 'hello'; allowed = @('world', 'planet') }
        @{ type = 'int'; value = 42; allowed = @(43, 44) }
    ) {
        param($type, $value, $allowed)

        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: $type
                allowedValues: $($allowed | ConvertTo-Json -Compress)
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $null = $config_yaml | dsc config -p $params_json get
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
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: $type
                ${constraint}: 3
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[parameters(''param1'')]'
"@
        $params_json = @{ parameters = @{ param1 = $value }} | ConvertTo-Json

        $null = $config_yaml | dsc config -p $params_json get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 4
    }

    It 'Default value is used when not provided' {
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            parameters:
              param1:
                type: string
                defaultValue: 'hello'
              param2:
                type: int
                defaultValue: 7
              param3:
                type: bool
                defaultValue: false
              param4:
                type: array
                defaultValue: ['hello', 'world']
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                text: '[concat(parameters(''param1''),'','',parameters(''param2''),'','',parameters(''param3''),'','',parameters(''param4''))]'
"@

        $out = $config_yaml | dsc config get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.text | Should -BeExactly 'hello,7,false,["hello","world"]'
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
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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

      $out = dsc -i $config_yaml config -p $params test | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results[0].result.actualState.family | Should -BeExactly $os
      $out.results[0].result.inDesiredState | Should -BeTrue
    }
}
