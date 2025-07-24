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
        @{ text = "[parameters('test').index]"; expected = '1' }
        @{ text = "[parameters('test').objectArray[parameters('test').index].name]"; expected = 'two' }
    ) {
        param($text, $expected)
        $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  test:
    type: object
    defaultValue:
      index: 1
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
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "$text"
"@
        $debug = $yaml | dsc -l trace config get -o yaml -f - 2>&1 | Out-String
        $out = $yaml | dsc config get -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because $debug
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
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "$expression"
"@
        $out = dsc config get -i $yaml 2>&1
        $LASTEXITCODE | Should -Be 2
        $out | Should -BeLike "*ERROR*"
    }

    It 'Multi-line string literals work' {
      $yamlPath = "$PSScriptRoot/../examples/multiline.dsc.yaml"
      $out = dsc config get -f $yamlPath | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results[0].result.actualState.output | Should -BeExactly @"
This is a
'multi-line'
string.

"@.Replace("`r", "")
      $out.results[1].result.actualState.output | Should -BeExactly "This is a single-quote: '"
    }

    It 'Nested Group resource does not invoke expressions' {
      $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Nested Group
  type: Microsoft.DSC/Group
  properties:
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
    - name: Deeply nested OSInfo
      type: Microsoft/OSInfo
      properties: {}
    - name: Deeply nested echo
      type: Microsoft.DSC.Debug/Echo
      properties:
        output:  >-
          [reference(
            resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')
          )]
      dependsOn:
        - "[resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')]"
'@

      $out = dsc config get -i $yaml | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0
      $out.results[0].result[1].result.actualState.output.family | Should -BeExactly $out.results[0].result[0].result.actualState.family
    }

    It 'Logical functions work: <expression>' -TestCases @(
        @{ expression = "[equals('a', 'a')]"; expected = $true }
        @{ expression = "[equals('a', 'b')]"; expected = $false }
        @{ expression = "[not(equals('a', 'b'))]"; expected = $true }
        @{ expression = "[greater(5, 3)]"; expected = $true }
        @{ expression = "[greater(3, 5)]"; expected = $false }
        @{ expression = "[greater(5, 5)]"; expected = $false }
        @{ expression = "[greaterOrEquals(5, 3)]"; expected = $true }
        @{ expression = "[greaterOrEquals(3, 5)]"; expected = $false }
        @{ expression = "[greaterOrEquals(5, 5)]"; expected = $true }
        @{ expression = "[less(3, 5)]"; expected = $true }
        @{ expression = "[less(5, 3)]"; expected = $false }
        @{ expression = "[less(5, 5)]"; expected = $false }
        @{ expression = "[lessOrEquals(3, 5)]"; expected = $true }
        @{ expression = "[lessOrEquals(5, 3)]"; expected = $false }
        @{ expression = "[lessOrEquals(5, 5)]"; expected = $true }
        @{ expression = "[greater('b', 'a')]"; expected = $true }
        @{ expression = "[greater('a', 'b')]"; expected = $false }
        @{ expression = "[greater('A', 'a')]"; expected = $false }
        @{ expression = "[greaterOrEquals('b', 'a')]"; expected = $true }
        @{ expression = "[greaterOrEquals('a', 'b')]"; expected = $false }
        @{ expression = "[greaterOrEquals('a', 'a')]"; expected = $true }
        @{ expression = "[greaterOrEquals('Aa', 'aa')]"; expected = $false }
        @{ expression = "[less('a', 'b')]"; expected = $true }
        @{ expression = "[less('b', 'a')]"; expected = $false }
        @{ expression = "[less('A', 'a')]"; expected = $true }
        @{ expression = "[lessOrEquals('a', 'b')]"; expected = $true }
        @{ expression = "[lessOrEquals('b', 'a')]"; expected = $false }
        @{ expression = "[lessOrEquals('a', 'a')]"; expected = $true }
        @{ expression = "[lessOrEquals('aa', 'Aa')]"; expected = $false }
        @{ expression = "[and(true, true)]"; expected = $true }
        @{ expression = "[and(true, false)]"; expected = $false }
        @{ expression = "[or(false, true)]"; expected = $true }
        @{ expression = "[or(false, false)]"; expected = $false }
        @{ expression = "[not(true)]"; expected = $false }
        @{ expression = "[not(or(true, false))]"; expected = $false }
        @{ expression = "[bool('TRUE')]" ; expected = $true }
        @{ expression = "[bool('False')]" ; expected = $false }
        @{ expression = "[bool(1)]" ; expected = $true }
        @{ expression = "[not(bool(0))]" ; expected = $true }
        @{ expression = "[coalesce(null, 'hello')]" ; expected = 'hello' }
        @{ expression = "[coalesce(null, null, 42)]" ; expected = 42 }
        @{ expression = "[coalesce(null, true)]" ; expected = $true }
        @{ expression = "[coalesce('first', 'second')]" ; expected = 'first' }
        @{ expression = "[true()]" ; expected = $true }
        @{ expression = "[false()]" ; expected = $false }
    ) {
        param($expression, $expected)
        $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "$expression"
"@
        $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
        $out.results[0].result.actualState.output | Should -Be $expected -Because ($out | ConvertTo-Json -Depth 10| Out-String)
    }

    It 'Comparison functions handle type mismatches: <expression>' -TestCases @(
        @{ expression = "[greater('a', 1)]" }
        @{ expression = "[greaterOrEquals('5', 3)]" }
        @{ expression = "[less(1, 'b')]" }
        @{ expression = "[lessOrEquals(5, 'a')]" }
    ) {
        param($expression)
        $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "$expression"
"@
        $out = dsc config get -i $yaml 2>$TestDrive/error.log
        $LASTEXITCODE | Should -Be 2
        $log = Get-Content -Path $TestDrive/error.log -Raw
        $log | Should -BeLike "*ERROR* Arguments must be of the same type*"
        
    }
}
