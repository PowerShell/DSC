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

  It 'Comparison functions work: <expression>' -TestCases @(
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
    @{ expression = "[coalesce('DSC', 'World')]" ; expected = 'DSC' }
    @{ expression = "[coalesce(42, 'fallback')]" ; expected = 42 }
    @{ expression = "[coalesce(true, false)]" ; expected = $true }
    @{ expression = "[coalesce('first', 'second')]" ; expected = 'first' }
    @{ expression = "[coalesce(createArray('a', 'b'), createArray('c', 'd'))]" ; expected = @('a', 'b') }
    @{ expression = "[coalesce(null(), 'fallback')]" ; expected = 'fallback' }

    @{ expression = "[coalesce(null(), createArray(1, 2, 3))]" ; expected = @(1, 2, 3) }
    @{ expression = "[coalesce(null(), null(), null(), 'finalValue')]" ; expected = 'finalValue' }
    @{ expression = "[coalesce(null(), 42, 'not-reached')]" ; expected = 42 }
    @{ expression = "[coalesce(null(), true, false)]" ; expected = $true }
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

  It 'Object functions work: <expression>' -TestCases @(
    @{ expression = "[createObject('name', 'test')]" ; expected = @{name = 'test' } }
    @{ expression = "[createObject('key1', 'value1', 'key2', 42)]" ; expected = @{key1 = 'value1'; key2 = 42 } }
    @{ expression = "[createObject()]" ; expected = @{} }
    @{ expression = "[null()]" ; expected = $null }
    @{ expression = "[createObject('key', null())]" ; expected = @{key = $null } }
    @{ expression = "[createObject('result', coalesce(null(), 'fallback'))]" ; expected = @{result = 'fallback' } }
    @{ expression = "[createObject('obj', coalesce(null(), createObject('name', 'test')))]" ; expected = @{obj = @{name = 'test' } } }
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
    foreach ($key in $out.results[0].result.actualState.output.psobject.properties.Name) {
      if ($out.results[0].result.actualState.output.$key -is [psobject]) {
        $out.results[0].result.actualState.output.$key.psobject.properties.value | Should -Be $expected.$key.values -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
      } else {
        $out.results[0].result.actualState.output.$key | Should -Be $expected.$key -Because ($out | ConvertTo-Json -Depth 10 | Out-String)  
      }
    }
  }

  It 'Logical functions work: <expression>' -TestCases @(
    @{ expression = "[equals('a', 'a')]"; expected = $true }
    @{ expression = "[equals('a', 'b')]"; expected = $false }
    @{ expression = "[not(equals('a', 'b'))]"; expected = $true }
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
    $out.results[0].result.actualState.output | Should -Be $expected -Because ($out | ConvertTo-Json -Depth 10 | Out-String)
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

  Context 'Resource name expression evaluation' {
    It 'Simple parameter expression in resource name: <expression>' -TestCases @(
      @{ expression = "[parameters('resourceName')]"; paramValue = 'TestResource'; expected = 'TestResource' }
      @{ expression = "[parameters('serviceName')]"; paramValue = 'MyService'; expected = 'MyService' }
    ) {
      param($expression, $paramValue, $expected)
      $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  resourceName:
    type: string
    defaultValue: $paramValue
  serviceName:
    type: string
    defaultValue: $paramValue
resources:
- name: "$expression"
  type: Microsoft/OSInfo
  properties: {}
"@
      $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
      $out.results[0].name | Should -Be $expected
    }

    It 'Concat function in resource name: <expression>' -TestCases @(
      @{ expression = "[concat('prefix-', parameters('name'))]"; paramValue = 'test'; expected = 'prefix-test' }
      @{ expression = "[concat(parameters('prefix'), '-', parameters('suffix'))]"; expected = 'start-end' }
      @{ expression = "[concat('Resource-', string(parameters('index')))]"; expected = 'Resource-42' }
    ) {
      param($expression, $paramValue, $expected)
      $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  name:
    type: string
    defaultValue: ${paramValue}
  prefix:
    type: string
    defaultValue: start
  suffix:
    type: string
    defaultValue: end
  index:
    type: int
    defaultValue: 42
resources:
- name: "$expression"
  type: Microsoft/OSInfo
  properties: {}
"@
      $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
      $out.results[0].name | Should -Be $expected
    }

    It 'Format function in resource name: <expression>' -TestCases @(
      @{ expression = "[format('Service-{0}', parameters('id'))]"; expected = 'Service-123' }
      @{ expression = "[format('{0}-{1}-{2}', parameters('env'), parameters('app'), parameters('ver'))]"; expected = 'prod-web-v1' }
    ) {
      param($expression, $expected)
      $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  id:
    type: string
    defaultValue: '123'
  env:
    type: string
    defaultValue: prod
  app:
    type: string
    defaultValue: web
  ver:
    type: string
    defaultValue: v1
  num:
    type: int
    defaultValue: 5
resources:
- name: "$expression"
  type: Microsoft/OSInfo
  properties: {}
"@
      $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
      $out.results[0].name | Should -Be $expected
    }

    It 'Complex expression in resource name: <expression>' -TestCases @(
      @{ expression = "[concat(parameters('prefix'), '-', string(add(parameters('base'), parameters('offset'))))]"; expected = 'server-105' }
      @{ expression = "[format('{0}-{1}', parameters('type'), if(equals(parameters('env'), 'prod'), 'production', 'development'))]"; expected = 'web-production' }

    ) {
      param($expression, $expected)
      $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  prefix:
    type: string
    defaultValue: server
  base:
    type: int
    defaultValue: 100
  offset:
    type: int
    defaultValue: 5
  type:
    type: string
    defaultValue: web
  env:
    type: string
    defaultValue: prod
  region:
    type: string
    defaultValue: EASTUS
  service:
    type: string
    defaultValue: WebApp
resources:
- name: "$expression"
  type: Microsoft/OSInfo
  properties: {}
"@
      $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
      $out.results[0].name | Should -Be $expected
    }

    It 'Expression with object parameter access: <expression>' -TestCases @(
      @{ expression = "[parameters('config').name]"; expected = 'MyApp' }
      @{ expression = "[concat(parameters('config').prefix, '-', parameters('config').id)]"; expected = 'app-001' }
      @{ expression = "[parameters('servers')[0]]"; expected = 'web01' }
      @{ expression = "[parameters('servers')[parameters('config').index]]"; expected = 'db01' }
    ) {
      param($expression, $expected)
      $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  config:
    type: object
    defaultValue:
      name: MyApp
      prefix: app
      id: '001'
      index: 1
  servers:
    type: array
    defaultValue:
      - web01
      - db01
      - cache01
resources:
- name: "$expression"
  type: Microsoft/OSInfo
  properties: {}
"@
      $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
      $out.results[0].name | Should -Be $expected
    }

    It 'Resource name expression error cases: <expression>' -TestCases @(
      @{ expression = "[parameters('nonexistent')]"; errorPattern = "*Parameter 'nonexistent' not found*" }
      @{ expression = "[concat()]"; errorPattern = "*requires at least 2 arguments*" }
      @{ expression = "[add('text', 'more')]"; errorPattern = "*Function 'add' does not accept string arguments, accepted types are: Number*" }
      @{ expression = "[parameters('config').nonexistent]"; errorPattern = "*Parser: Member 'nonexistent' not found*" }
      @{ expression = "[parameters('array')[10]]"; errorPattern = "*Parser: Index is out of bounds*" }
    ) {
      param($expression, $errorPattern)
      $yaml = @"
`$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  config:
    type: object
    defaultValue:
      name: test
  array:
    type: array
    defaultValue:
      - item1
      - item2
resources:
- name: "$expression"
  type: Microsoft/OSInfo
  properties: {}
"@
      dsc config get -i $yaml 2>$TestDrive/error.log | Out-Null
      $LASTEXITCODE | Should -Be 2
      $errorLog = Get-Content $TestDrive/error.log -Raw
      $errorLog | Should -BeLike $errorPattern
    }

    It 'Resource name expression must evaluate to string' {
      $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  number:
    type: int
    defaultValue: 42
resources:
- name: "[parameters('number')]"
  type: Microsoft/OSInfo
  properties: {}
'@
      dsc config get -i $yaml 2>$TestDrive/error.log | Out-Null
      $LASTEXITCODE | Should -Be 2
      $errorLog = Get-Content $TestDrive/error.log -Raw
      $errorLog | Should -BeLike "*Resource name result is not a string*"
    }

    It 'Resource name expression with conditional logic' {
      $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  isProd:
    type: bool
    defaultValue: true
  serviceName:
    type: string
    defaultValue: api
resources:
- name: "[concat(parameters('serviceName'), if(parameters('isProd'), '-prod', '-dev'))]"
  type: Microsoft/OSInfo
  properties: {}
'@
      $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
      $out.results[0].name | Should -Be 'api-prod'
    }

    It 'Resource name with nested function calls' {
      $yaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  config:
    type: object
    defaultValue:
      services:
        - web
        - api
        - db
      selectedIndex: 1
resources:
- name: "[concat('SERVICE-', parameters('config').services[parameters('config').selectedIndex])]"
  type: Microsoft/OSInfo
  properties: {}
'@
      $out = dsc config get -i $yaml 2>$TestDrive/error.log | ConvertFrom-Json
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw | Out-String)
      $out.results[0].name | Should -Be 'SERVICE-api'
    }
  }
}
