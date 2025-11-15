# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'map() function with lambda tests' {
    It 'map with simple lambda multiplies each element by 2' {
        $config_yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  numbers:
    type: array
    defaultValue: [1, 2, 3]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(parameters('numbers'), lambda('x', mul(lambdaVariables('x'), 2)))]"
'@
        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -Be @(2,4,6)
    }

    It 'map with lambda using index parameter' {
        $config_yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  items:
    type: array
    defaultValue: [10, 20, 30]
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(parameters('items'), lambda('val', 'i', add(lambdaVariables('val'), lambdaVariables('i'))))]"
'@
        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -Be @(10,21,32)
    }

    It 'map with range generates array' {
        $config_yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(range(0, 3), lambda('x', mul(lambdaVariables('x'), 3)))]"
'@
        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -Be @(0,3,6)
    }

    It 'map returns empty array for empty input' {
        $config_yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[map(createArray(), lambda('x', mul(lambdaVariables('x'), 2)))]"
'@
        $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output.Count | Should -Be 0
    }
}
