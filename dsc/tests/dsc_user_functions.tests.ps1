# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'user function tests' {
    It 'user function working with expression: <expression>' -TestCases @(
        @{ expression = "[MyFunction.ComboFunction('test', 42, true)]"; expected = 'test-42-True' }
        @{ expression = "[MyOtherNamespace.ArrayFunction(createArray('a','b','c','d'))]"; expected = @('["b","c","d"]-a') }
    ) {
        param($expression, $expected)

        $configYaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: MyFunction
  members:
    ComboFunction:
      parameters:
      - name: StringParam
        type: string
      - name: NumberParam
        type: int
      - name: BoolParam
        type: bool
      output:
        type: string
        value: "[format('{0}-{1}-{2}', parameters('StringParam'), parameters('NumberParam'), parameters('BoolParam'))]"
- namespace: MyOtherNamespace
  members:
    ArrayFunction:
      parameters:
      - name: ArrayParam
        type: array
      output:
        type: array
        value: "[array(format('{0}-{1}', string(skip(parameters('ArrayParam'),1)), first(parameters('ArrayParam'))))]"
resources:
- name: test
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "$expression"
"@

        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $testdrive/error.log | Out-String)
        $out.results[0].result.actualState.output | Should -Be $expected -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
    }

    It 'user function returning object works' {
        $configYaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: MyObjectFunction
  members:
    ObjectFunction:
      parameters:
      - name: ObjectParam
        type: object
      output:
        type: object
        value: "[createObject('myKey', concat('#', string(parameters('ObjectParam'))))]"
resources:
- name: test
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[MyObjectFunction.ObjectFunction(createObject('key','value'))]"
"@

        $out = dsc -l trace config get -i $configYaml 2>$testdrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $testdrive/error.log | Out-String)
        $out.results[0].result.actualState.output.myKey | Should -Be '#{"key":"value"}' -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
    }

    It 'user functions cannot call function with expression: <expression>' -TestCases @(
        @{ expression = "[reference('foo/bar')]"; errorText = "The 'reference()' function is not available in user-defined functions" }
        @{ expression = "[utcNow()]"; errorText = "The 'utcNow()' function can only be used as a parameter default" }
        @{ expression = "[variables('myVar')]"; errorText = "The 'variables()' function is not available in user-defined functions" }
        @{ expression = "[MyFunction.OtherFunction()]"; errorText = "Unknown user function 'MyFunction.OtherFunction'" }
        @{ expression = "[MyFunction.BadFunction()]"; errorText = "Unknown user function 'MyFunction.BadFunction'" }
    ) {
        param($expression, $errorText)

        $configYaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
variables:
  myVar: someValue
functions:
- namespace: MyFunction
  members:
    BadFunction:
      output:
        type: string
        value: "$expression"
    OtherFunction:
      output:
        type: string
        value: "test"
resources:
- name: test
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[MyFunction.BadFunction()]"
"@

        dsc -l trace config get -i $configYaml 2>$testdrive/error.log | Out-Null
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content $testdrive/error.log | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -BeLike "*$errorText*" -Because (Get-Content $testdrive/error.log | Out-String)
    }

    It 'user function with invalid parameter fails' {
        $configYaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: MyFunction
  members:
    BadFunction:
      parameters:
      - name: Param1
        type: string
      output:
        type: string
        value: "[parameters('BadParam')]"
resources:
- name: test
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[MyFunction.BadFunction('test')]"
"@

        dsc -l trace config get -i $configYaml 2>$testdrive/error.log | Out-Null
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content $testdrive/error.log | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -BeLike "*Parameter 'BadParam' not found in context*" -Because (Get-Content $testdrive/error.log | Out-String)
    }

    It 'user function with wrong output type fails' {
        $configYaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: MyFunction
  members:
    BadFunction:
      output:
        type: int
        value: "'this is a string'"
resources:
- name: test
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[MyFunction.BadFunction()]"
"@
        dsc -l trace config get -i $configYaml 2>$testdrive/error.log | Out-Null
        $LASTEXITCODE | Should -Be 2 -Because (Get-Content $testdrive/error.log | Out-String)
        (Get-Content $testdrive/error.log -Raw) | Should -BeLike "*Output of user function 'MyFunction.BadFunction' returned an integer, but was expected to be of type 'int'*" -Because (Get-Content $testdrive/error.log | Out-String)
    }
}
