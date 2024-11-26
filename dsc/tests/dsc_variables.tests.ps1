# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Configruation variables tests' {
    It 'Variables example config works' {
        $configFile = "$PSSCriptRoot/../examples/variables.dsc.yaml"
        $out = dsc config get -f $configFile | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -BeExactly 'myOutput is: Hello world!, myObject is: baz'
    }

    It 'Duplicated variable takes last value' {
        $configYaml = @'
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
variables:
  myVariable: foo
  myVariable: bar
resources:
- name: test
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[variables('myVariable')]"
'@
        $out = dsc config get -i $configYaml | ConvertFrom-Json
        Write-Verbose -Verbose $out
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -Be 'bar'
    }

    It 'Missing variable returns error' {
        $configYaml = @'
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
variables:
  hello: world
resources:
- name: test
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[variables('myVariable')]"
'@
        $out = dsc config get -i $configYaml 2>&1 | Out-String
        Write-Verbose -Verbose $out
        $LASTEXITCODE | Should -Be 2
        $out | Should -BeLike "*Variable 'myVariable' does not exist or has not been initialized yet*"
    }
}
