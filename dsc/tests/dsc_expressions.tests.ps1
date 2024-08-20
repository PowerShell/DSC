# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Expressions tests' {
    It 'Accessors work: <text>' -TestCases @(
        @{ text = "[parameters('test').hello]"; expected = '@{world=there}' }
        @{ text = "[parameters('test').hello.world]"; expected = 'there' }
        @{ text = "[parameters('test').array[0]]"; expected = 'one' }
        @{ text = "[parameters('test').array[1][1]]"; expected = 'three' }
        @{ text = "[parameters('test').objectArray[0].name]"; expected = 'one' }
        @{ text = "[parameters('test').objectArray[1].value[0]]"; expected = '2' }
        @{ text = "[parameters('test').objectArray[1].value[1].name]"; expected = 'three' }
    ) {
        param($text, $expected)
        $yaml = @"
`$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
parameters:
  test:
    type: object
    defaultValue:
      hello:
        world: there
      array:
      - one
      - [ 'two', 'three' ]
      objectArray:
      - name: one
        value: 1
      - name: two
        value:
        - 2
        - nestedObject:
          name: three
          value: 3
resources:
- name: echo
  type: Test/Echo
  properties:
    output: "$text"
"@
        $debug = $yaml | dsc -l debug config get -f yaml 2>&1 | Out-String
        $out = $yaml | dsc config get | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.actualState.output | Should -Be $expected -Because $debug
    }

    It 'Invalid expressions: <expression>' -TestCases @(
        @{ expression = "[concat('A','B')].hello" }
        @{ expression = "[concat('A','B')](0)" }
        @{ expression = "[concat('a','b').hello]" }
        @{ expression = "[concat('a','b')[0]]" }
    ) {
        param($expression)
        $yaml = @"
`$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: echo
  type: Test/Echo
  properties:
    output: "$expression"
"@
        $out = dsc config get -d $yaml 2>&1
        $LASTEXITCODE | Should -Be 2
        $out | Should -BeLike "*ERROR*"
    }

    It 'Multi-line string literals work' {
      $yamlPath = "$PSScriptRoot/../examples/multiline.dsc.yaml"
      $out = dsc config get -p $yamlPath | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results[0].result.actualState.output | Should -BeExactly @"
This is a
multi-line
string.

"@
    }
}
