# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for function expressions' {
  It 'function works: <text>' -TestCases @(
    @{ text = "[concat('a', 'b')]"; expected = 'ab' }
    @{ text = "[concat('a', 'b', 'c')]"; expected = 'abc' }
    @{ text = "[concat('a', concat('b', 'c'))]"; expected = 'abc' }
    @{ text = "[base64('ab')]"; expected = 'YWI=' }
    @{ text = "[base64(concat('a','b'))]"; expected = 'YWI=' }
    @{ text = "[base64(base64(concat('a','b')))]"; expected = 'WVdJPQ==' }
  ) {
    param($text, $expected)

    $escapedText = $text -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedText'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'path(<path>) works' -TestCases @(
    @{ path = "systemRoot(), 'a'"; expected = "$PSHOME$([System.IO.Path]::DirectorySeparatorChar)a" }
    @{ path = "'a', 'b', 'c'"; expected = "a$([System.IO.Path]::DirectorySeparatorChar)b$([System.IO.Path]::DirectorySeparatorChar)c" }
  ) {
    param($path, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[path($path)]"
"@
    $out = $config_yaml | dsc config --system-root $PSHOME get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'default systemRoot() is correct for the OS' {
    $config_yaml = @'
            $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[systemRoot()]"
'@

    $expected = if ($IsWindows) {
      $env:SYSTEMDRIVE + '\'
    } else {
      '/'
    }
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'union function works for: <expression>' -TestCases @(
    @{ expression = "[union(parameters('firstArray'), parameters('secondArray'))]"; expected = @('ab', 'cd', 'ef') }
    @{ expression = "[union(parameters('firstObject'), parameters('secondObject'))]"; expected = [pscustomobject]@{ one = 'a'; two = 'c'; three = 'd' } }
    @{ expression = "[union(parameters('secondArray'), parameters('secondArray'))]"; expected = @('cd', 'ef') }
    @{ expression = "[union(parameters('secondObject'), parameters('secondObject'))]"; expected = [pscustomobject]@{ two = 'c'; three = 'd' } }
    @{ expression = "[union(parameters('firstObject'), parameters('firstArray'))]"; isError = $true }
  ) {
    param($expression, $expected, $isError)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              firstObject:
                type: object
                defaultValue:
                  one: a
                  two: b
              secondObject:
                type: object
                defaultValue:
                  two: c
                  three: d
              firstArray:
                type: array
                defaultValue:
                - ab
                - cd
              secondArray:
                type: array
                defaultValue:
                - cd
                - ef
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    if ($isError) {
      $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.log -Raw)
      (Get-Content $TestDrive/error.log -Raw) | Should -Match 'All arguments must either be arrays or objects'
    } else {
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
      ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
    }
  }

  It 'intersection function works for: <expression>' -TestCases @(
    @{ expression = "[intersection(parameters('firstArray'), parameters('secondArray'))]"; expected = @('cd') }
    @{ expression = "[intersection(parameters('firstObject'), parameters('secondObject'))]"; expected = [pscustomobject]@{ two = 'b' } }
    @{ expression = "[intersection(parameters('thirdArray'), parameters('fourthArray'))]"; expected = @('ef', 'gh') }
    @{ expression = "[intersection(parameters('thirdObject'), parameters('fourthObject'))]"; expected = [pscustomobject]@{ three = 'd' } }
    @{ expression = "[intersection(parameters('firstArray'), parameters('thirdArray'))]"; expected = @() }
    @{ expression = "[intersection(parameters('firstObject'), parameters('firstArray'))]"; isError = $true }
    @{ expression = "[intersection(parameters('firstArray'), parameters('secondArray'), parameters('fifthArray'))]"; expected = @('cd') }
    @{ expression = "[intersection(parameters('firstObject'), parameters('secondObject'), parameters('sixthObject'))]"; expected = [pscustomobject]@{ two = 'b' } }
    @{ expression = "[intersection(parameters('nestedObject1'), parameters('nestedObject2'))]"; expected = [pscustomobject]@{
        shared = [pscustomobject]@{ value = 42; flag = $true }
        level  = 1
      }
    }
    @{ expression = "[intersection(parameters('nestedObject1'), parameters('nestedObject3'))]"; expected = [pscustomobject]@{ level = 1 } }
    @{ expression = "[intersection(parameters('nestedObject1'), parameters('nestedObject2'), parameters('nestedObject4'))]"; expected = [pscustomobject]@{ level = 1 } }
  ) {
    param($expression, $expected, $isError)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              firstObject:
                type: object
                defaultValue:
                  one: a
                  two: b
              secondObject:
                type: object
                defaultValue:
                  two: b
                  three: d
              thirdObject:
                type: object
                defaultValue:
                  two: c
                  three: d
              fourthObject:
                type: object
                defaultValue:
                  three: d
                  four: e
              sixthObject:
                type: object
                defaultValue:
                  two: b
                  five: f
              nestedObject1:
                type: object
                defaultValue:
                  shared:
                    value: 42
                    flag: true
                  level: 1
                  unique1: test
              nestedObject2:
                type: object
                defaultValue:
                  shared:
                    value: 42
                    flag: true
                  level: 1
                  unique2: test
              nestedObject3:
                type: object
                defaultValue:
                  shared:
                    value: 24
                    flag: true
                  level: 1
                  unique3: test
              nestedObject4:
                type: object
                defaultValue:
                  level: 1
                  different:
                    value: 100
                    flag: false
              firstArray:
                type: array
                defaultValue:
                - ab
                - cd
              secondArray:
                type: array
                defaultValue:
                - cd
                - ef
              thirdArray:
                type: array
                defaultValue:
                - ef
                - gh
              fourthArray:
                type: array
                defaultValue:
                - gh
                - ef
                - ij
              fifthArray:
                type: array
                defaultValue:
                - cd
                - kl
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    if ($isError) {
      $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.log -Raw)
      (Get-Content $TestDrive/error.log -Raw) | Should -Match 'All arguments must either be arrays or objects'
    } else {
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
      ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
    }
  }

  It 'contain function works for: <expression>' -TestCases @(
    @{ expression = "[contains(parameters('array'), 'a')]" ; expected = $true }
    @{ expression = "[contains(parameters('array'), 2)]" ; expected = $false }
    @{ expression = "[contains(parameters('array'), 1)]" ; expected = $true }
    @{ expression = "[contains(parameters('array'), 'z')]" ; expected = $false }
    @{ expression = "[contains(parameters('object'), 'a')]" ; expected = $true }
    @{ expression = "[contains(parameters('object'), 'c')]" ; expected = $false }
    @{ expression = "[contains(parameters('object'), 3)]" ; expected = $true }
    @{ expression = "[contains(parameters('object'), parameters('object'))]" ; isError = $true }
    @{ expression = "[contains(parameters('array'), parameters('array'))]" ; isError = $true }
    @{ expression = "[contains(parameters('string'), 'not found')]" ; expected = $false }
    @{ expression = "[contains(parameters('string'), 'hello')]" ; expected = $true }
    @{ expression = "[contains(parameters('string'), 12)]" ; expected = $true }
  ) {
    param($expression, $expected, $isError)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              array:
                type: array
                defaultValue:
                - a
                - b
                - 0
                - 1
              object:
                type: object
                defaultValue:
                  a: 1
                  b: 2
                  3: c
              string:
                type: string
                defaultValue: 'hello 123 world!'
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    if ($isError) {
      $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.log -Raw)
      (Get-Content $TestDrive/error.log -Raw) | Should -Match 'accepted types are: String, Number'
    } else {
      $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
      ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
    }
  }

  It 'length function works for: <expression>' -TestCases @(
    @{ expression = "[length(parameters('array'))]" ; expected = 3 }
    @{ expression = "[length(parameters('object'))]" ; expected = 4 }
    @{ expression = "[length(parameters('string'))]" ; expected = 12 }
    @{ expression = "[length('')]"; expected = 0 }
  ) {
    param($expression, $expected, $isError)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              array:
                type: array
                defaultValue:
                - a
                - b
                - c
              object:
                type: object
                defaultValue:
                  one: a
                  two: b
                  three: c
                  four: d
              string:
                type: string
                defaultValue: 'hello world!'
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'empty function works for: <expression>' -TestCases @(
    @{ expression = "[empty(parameters('array'))]" ; expected = $false }
    @{ expression = "[empty(parameters('object'))]" ; expected = $false }
    @{ expression = "[empty(parameters('string'))]" ; expected = $false }
    @{ expression = "[empty(parameters('emptyArray'))]" ; expected = $true }
    @{ expression = "[empty(parameters('emptyObject'))]" ; expected = $true }
    @{ expression = "[empty('')]" ; expected = $true }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              array:
                type: array
                defaultValue:
                - a
                - b
                - c
              emptyArray:
                type: array
                defaultValue: []
              object:
                type: object
                defaultValue:
                  one: a
                  two: b
                  three: c
              emptyObject:
                type: object
                defaultValue: {}
              string:
                type: string
                defaultValue: 'hello world!'
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'utcNow function works for: utcNow(<format>)' -TestCases @(
    @{ format = $null; regex = '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z$' }
    @{ format = "yyyy-MM-dd"; regex = '^\d{4}-\d{2}-\d{2}$' }
    @{ format = "yyyy-MM-ddTHH"; regex = '^\d{4}-\d{2}-\d{2}T\d{2}$' }
    @{ format = "yyyy-MM-ddTHHZ"; regex = '^\d{4}-\d{2}-\d{2}T\d{2}Z$' }
    @{ format = "MMM dd, yyyy HH"; regex = '^(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) \d{2}, \d{4} \d{2}$' }
    @{ format = "yy-MMMM-dddd tt H"; regex = '^\d{2}-(January|February|March|April|May|June|July|August|September|October|November|December)-(Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday) (AM|PM) \d+$' }
    @{ format = "MMM ddd zzz"; regex = '^(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) (Sun|Mon|Tue|Wed|Thu|Fri|Sat) \+00:00$' }
    @{ format = "yy yyyy MM MMM MMMM"; regex = '^\d{2} \d{4} \d{2} (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) (January|February|March|April|May|June|July|August|September|October|November|December)$' }
    @{ format = "yyyy-MM-ddTHH:mm:ss"; regex = '^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$' }
  ) {
    param($format, $regex)

    if ($null -ne $format) {
      $format = "'$format'"
    }

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            parameters:
              test:
                type: string
                defaultValue: "[utcNow($format)]"
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parameters('test')]"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    # ConvertFrom-Json will convert the date to a DateTime object, so we use regex to capture the string
    $out -match '"output":"(?<date>.*?)"' | Should -BeTrue -Because "Output should contain a date"
    $actual = $matches['date']
    # compare against the regex
    $actual | Should -Match $regex -Because "Output date '$actual' should match regex '$regex'"
  }

  It 'utcNow errors if used not as a parameter default' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[utcNow()]"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 2 -Because (Get-Content $TestDrive/error.log -Raw)
    $out | Should -BeNullOrEmpty -Because "Output should be null or empty"
    (Get-Content $TestDrive/error.log -Raw) | Should -Match "The 'utcNow\(\)' function can only be used as a parameter default"
  }

  It 'uniqueString function works for: <expression>' -TestCases @(
    @{ expression = "[uniqueString('a')]" ; expected = 'cfvwxu6sc4lqo' }
    @{ expression = "[uniqueString('a', 'b', 'c')]" ; expected = 'bhw7m6t6ntwd6' }
    @{ expression = "[uniqueString('a', 'b', 'c', 'd')]" ; expected = 'yxzg7ur4qetcy' }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'string function works for: <expression>' -TestCases @(
    @{ expression = "[string('hello')]"; expected = 'hello' }
    @{ expression = "[string(123)]"; expected = '123' }
    @{ expression = "[string(true)]"; expected = 'true' }
    @{ expression = "[string(null())]"; expected = 'null' }
    @{ expression = "[string(createArray('a', 'b'))]"; expected = '["a","b"]' }
    @{ expression = "[string(createObject('a', 1))]"; expected = '{"a":1}' }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'array function works for: <expression>' -TestCases @(
    @{ expression = "[array('hello')]"; expected = @('hello') }
    @{ expression = "[array(42)]"; expected = @(42) }
    @{ expression = "[array(createObject('key', 'value'))]"; expected = @([pscustomobject]@{ key = 'value' }) }
    @{ expression = "[array(createArray('a', 'b'))]"; expected = @(@('a', 'b')) }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'first function works for: <expression>' -TestCases @(
    @{ expression = "[first(createArray('hello', 'world'))]"; expected = 'hello' }
    @{ expression = "[first(createArray(1, 2, 3))]"; expected = 1 }
    @{ expression = "[first('hello')]"; expected = 'h' }
    @{ expression = "[first('a')]"; expected = 'a' }
    @{ expression = "[first(array('mixed'))]"; expected = 'mixed' }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'last function works for: <expression>' -TestCases @(
    @{ expression = "[last(createArray('hello', 'world'))]"; expected = 'world' }
    @{ expression = "[last(createArray(1, 2, 3))]"; expected = 3 }
    @{ expression = "[last('hello')]"; expected = 'o' }
    @{ expression = "[last('a')]"; expected = 'a' }
    @{ expression = "[last(array('mixed'))]"; expected = 'mixed' }
    @{ expression = "[last(createArray())]"; expected = $null }
    @{ expression = "[last('')]"; expected = '' }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'indexOf function works for: <expression>' -TestCases @(
    @{ expression = "[indexOf(createArray('apple', 'banana', 'cherry'), 'banana')]"; expected = 1 }
    @{ expression = "[indexOf(createArray('apple', 'banana', 'cherry'), 'cherry')]"; expected = 2 }
    @{ expression = "[indexOf(createArray(10, 20, 30), 20)]"; expected = 1 }
    @{ expression = "[indexOf(createArray('a', 'b', 'a', 'c'), 'a')]"; expected = 0 }
    @{ expression = "[indexOf(createArray('apple', 'banana'), 'orange')]"; expected = -1 }
    @{ expression = "[indexOf(createArray('Apple', 'Banana'), 'apple')]"; expected = -1 }
    @{ expression = "[indexOf(createArray(), 'test')]"; expected = -1 }
    @{ expression = "[indexOf(createArray(createArray('a', 'b'), createArray('c', 'd')), createArray('c', 'd'))]"; expected = 1 }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'join function works for: <expression>' -TestCases @(
    @{ expression = "[join(createArray('a','b','c'), '-')]"; expected = 'a-b-c' }
    @{ expression = "[join(createArray(), '-')]"; expected = '' }
    @{ expression = "[join(createArray(1,2,3), ',')]"; expected = '1,2,3' }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'skip function works for: <expression>' -TestCases @(
    @{ expression = "[skip(createArray('a','b','c','d'), 2)]"; expected = @('c', 'd') }
    @{ expression = "[skip('hello', 2)]"; expected = 'llo' }
    @{ expression = "[skip(createArray('a','b'), 0)]"; expected = @('a', 'b') }
    @{ expression = "[skip('abc', 0)]"; expected = 'abc' }
    @{ expression = "[skip(createArray('a','b'), 5)]"; expected = @() }
    @{ expression = "[skip('', 1)]"; expected = '' }
    # Negative counts are treated as zero
    @{ expression = "[skip(createArray('x','y'), -3)]"; expected = @('x', 'y') }
    @{ expression = "[skip('xy', -1)]"; expected = 'xy' }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'take function works for: <expression>' -TestCases @(
    @{ expression = "[take(createArray('a','b','c','d'), 2)]"; expected = @('a', 'b') }
    @{ expression = "[take('hello', 2)]"; expected = 'he' }
    @{ expression = "[take(createArray('a','b'), 0)]"; expected = @() }
    @{ expression = "[take('abc', 0)]"; expected = '' }
    @{ expression = "[take(createArray('a','b'), 5)]"; expected = @('a', 'b') }
    @{ expression = "[take('hi', 10)]"; expected = 'hi' }
    @{ expression = "[take('', 1)]"; expected = '' }
    @{ expression = "[take(createArray(), 2)]"; expected = @() }
    # Negative and zero counts return empty
    @{ expression = "[take(createArray('x','y','z'), -1)]"; expected = @() }
    @{ expression = "[take('hello', -2)]"; expected = '' }
    # Take all elements
    @{ expression = "[take(createArray('x','y','z'), 3)]"; expected = @('x', 'y', 'z') }
    @{ expression = "[take('test', 4)]"; expected = 'test' }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'lastIndexOf function works for: <expression>' -TestCases @(
    @{ expression = "[lastIndexOf(createArray('a', 'b', 'a', 'c'), 'a')]"; expected = 2 }
    @{ expression = "[lastIndexOf(createArray(10, 20, 30, 20), 20)]"; expected = 3 }
    @{ expression = "[lastIndexOf(createArray('Apple', 'Banana'), 'apple')]"; expected = -1 }
    @{ expression = "[lastIndexOf(createArray(createArray('a','b'), createArray('c','d'), createArray('a','b')), createArray('a','b'))]"; expected = 2 }
    @{ expression = "[lastIndexOf(createArray(createObject('name','John'), createObject('name','Jane'), createObject('name','John')), createObject('name','John'))]"; expected = 2 }
    @{ expression = "[lastIndexOf(createArray(), 'test')]"; expected = -1 }
    # Objects are compared by deep equality: same keys and values are equal, regardless of property order.
    # Both createObject('a',1,'b',2) and createObject('b',2,'a',1) are considered equal.
    # Therefore, lastIndexOf returns 1 (the last position where an equal object occurs).
    @{ expression = "[lastIndexOf(createArray(createObject('a',1,'b',2), createObject('b',2,'a',1)), createObject('a',1,'b',2))]"; expected = 1 }
    @{ expression = "[lastIndexOf(createArray('1','2','3'), 1)]"; expected = -1 }
    @{ expression = "[lastIndexOf(createArray(1,2,3), '1')]"; expected = -1 }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'context function works' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[context()]"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    $context = $out.results[0].result.actualState.output
    $os = osinfo | ConvertFrom-Json
    $context.os.family | Should -BeExactly $os.family
    $context.os.version | Should -BeExactly $os.version
    $context.os.bitness | Should -BeExactly $os.bitness
    $context.os.architecture | Should -BeExactly $os.architecture
    $context.security | Should -BeExactly $out.metadata.'Microsoft.DSC'.securityContext
  }

  It 'range function works: <expression>' -TestCases @(
    @{ expression = '[range(1, 3)]'; expected = @(1, 2, 3) }
    @{ expression = '[range(0, 5)]'; expected = @(0, 1, 2, 3, 4) }
    @{ expression = '[range(-2, 4)]'; expected = @(-2, -1, 0, 1) }
    @{ expression = '[range(10, 0)]'; expected = @() }
    @{ expression = '[range(100, 3)]'; expected = @(100, 101, 102) }
    @{ expression = '[first(range(2147473647, 10000))]'; expected = 2147473647 }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
  }

  It 'range function handles errors correctly: <expression>' -TestCases @(
    @{ expression = '[range(1, -1)]'; expectedError = 'Count must be non-negative' }
    @{ expression = '[range(1, 10001)]'; expectedError = 'Count must not exceed 10000' }
    @{ expression = '[range(2147483647, 1)]'; expectedError = 'Sum of startIndex and count must not exceed 2147483647' }
  ) {
    param($expression, $expectedError)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log
    $LASTEXITCODE | Should -Not -Be 0
    $errorContent = Get-Content $TestDrive/error.log -Raw
    $errorContent | Should -Match ([regex]::Escape($expectedError))
  }

  It 'substring function works for: <expression>' -TestCases @(
    @{ expression = "[substring('hello world', 6, 5)]"; expected = 'world' }
    @{ expression = "[substring('hello', 0, 2)]"; expected = 'he' }
    @{ expression = "[substring('hello', 1, 3)]"; expected = 'ell' }
    @{ expression = "[substring('hello', 2)]"; expected = 'llo' }
    @{ expression = "[substring('hello', 0)]"; expected = 'hello' }
    @{ expression = "[substring('hello', 5)]"; expected = '' }
    @{ expression = "[substring('hello', 1, 1)]"; expected = 'e' }
    @{ expression = "[substring('hello', 5, 0)]"; expected = '' }
    @{ expression = "[substring('', 0)]"; expected = '' }
    @{ expression = "[substring('', 0, 0)]"; expected = '' }
    @{ expression = "[substring('héllo', 1, 2)]"; expected = 'él' }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'substring function error handling: <expression>' -TestCases @(
    @{ expression = "[substring('hello', -1, 2)]"; expectedError = 'Start index cannot be negative' }
    @{ expression = "[substring('hello', 1, -1)]"; expectedError = 'Length cannot be negative' }
    @{ expression = "[substring('hello', 10, 1)]"; expectedError = 'Start index is beyond the end of the string' }
    @{ expression = "[substring('hello', 2, 10)]"; expectedError = 'Length extends beyond the end of the string' }
  ) {
    param($expression, $expectedError)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: `"$expression`"
"@
    $null = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log
    $LASTEXITCODE | Should -Not -Be 0
    $errorContent = Get-Content $TestDrive/error.log -Raw
    $errorContent | Should -Match ([regex]::Escape($expectedError))
  }

  It 'mixed booleans with functions works' -TestCases @(
    @{ expression = "[and(true(), false, not(false))]"; expected = $false }
    @{ expression = "[or(false, false(), not(false()))]"; expected = $true }
    @{ expression = "[and(true(), true, not(false))]"; expected = $true }
    @{ expression = "[or(false, false(), not(true()))]"; expected = $false }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'base64ToString function works for: <expression>' -TestCases @(
    @{ expression = "[base64ToString('aGVsbG8gd29ybGQ=')]"; expected = 'hello world' }
    @{ expression = "[base64ToString('')]"; expected = '' }
    @{ expression = "[base64ToString('aMOpbGxv')]"; expected = 'héllo' }
    @{ expression = "[base64ToString('eyJrZXkiOiJ2YWx1ZSJ9')]"; expected = '{"key":"value"}' }
    @{ expression = "[base64ToString(base64('test message'))]"; expected = 'test message' }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'base64ToString function error handling: <expression>' -TestCases @(
    @{ expression = "[base64ToString('invalid!@#')]" ; expectedError = 'Invalid base64 encoding' }
    @{ expression = "[base64ToString('/w==')]" ; expectedError = 'Decoded bytes do not form valid UTF-8' }
  ) {
    param($expression, $expectedError)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: `"$expression`"
"@
    $null = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log
    $LASTEXITCODE | Should -Not -Be 0
    $errorContent = Get-Content $TestDrive/error.log -Raw
    $errorContent | Should -Match $expectedError
  }

  It 'toUpper function works for: <expression>' -TestCases @(
    @{ expression = "[toUpper('hello world')]"; expected = 'HELLO WORLD' }
    @{ expression = "[toUpper('Hello World')]"; expected = 'HELLO WORLD' }
    @{ expression = "[toUpper('HELLO WORLD')]"; expected = 'HELLO WORLD' }
    @{ expression = "[toUpper('')]"; expected = '' }
    @{ expression = "[toUpper('Hello123!@#')]"; expected = 'HELLO123!@#' }
    @{ expression = "[toUpper('café')]"; expected = 'CAFÉ' }
    @{ expression = "[toUpper('  hello  world  ')]"; expected = '  HELLO  WORLD  ' }
    @{ expression = "[toUpper('a')]"; expected = 'A' }
    @{ expression = "[toUpper(concat('hello', ' world'))]"; expected = 'HELLO WORLD' }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'toLower function works for: <expression>' -TestCases @(
    @{ expression = "[toLower('HELLO WORLD')]"; expected = 'hello world' }
    @{ expression = "[toLower('Hello World')]"; expected = 'hello world' }
    @{ expression = "[toLower('hello world')]"; expected = 'hello world' }
    @{ expression = "[toLower('')]"; expected = '' }
    @{ expression = "[toLower('HELLO123!@#')]"; expected = 'hello123!@#' }
    @{ expression = "[toLower('CAFÉ')]"; expected = 'café' }
    @{ expression = "[toLower('  HELLO  WORLD  ')]"; expected = '  hello  world  ' }
    @{ expression = "[toLower('A')]"; expected = 'a' }
    @{ expression = "[toLower(concat('HELLO', ' WORLD'))]"; expected = 'hello world' }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'trim function works for: <expression>' -TestCases @(
    @{ expression = "[trim('   hello')]"; expected = 'hello' }
    @{ expression = "[trim('hello   ')]"; expected = 'hello' }
    @{ expression = "[trim('  hello world  ')]"; expected = 'hello world' }
    @{ expression = "[trim('hello')]"; expected = 'hello' }
    @{ expression = "[trim('')]"; expected = '' }
    @{ expression = "[trim('   ')]"; expected = '' }
    @{ expression = "[trim('  hello  world  ')]"; expected = 'hello  world' }
    @{ expression = "[trim('  café  ')]"; expected = 'café' }
    @{ expression = "[trim(' a ')]"; expected = 'a' }
    @{ expression = "[trim(concat('  hello', '  '))]"; expected = 'hello' }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'items function converts object to array: <expression>' -TestCases @(
    @{ expression = "[length(items(createObject('a', 1, 'b', 2)))]"; expected = 2 }
    @{ expression = "[length(items(createObject()))]"; expected = 0 }
    @{ expression = "[items(createObject('name', 'John'))[0].key]"; expected = 'name' }
    @{ expression = "[items(createObject('name', 'John'))[0].value]"; expected = 'John' }
    @{ expression = "[items(createObject('a', 1, 'b', 2, 'c', 3))[1].key]"; expected = 'b' }
    @{ expression = "[items(createObject('x', 'hello', 'y', 'world'))[0].value]"; expected = 'hello' }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'items function handles nested values: <expression>' -TestCases @(
    @{ expression = "[items(createObject('person', createObject('name', 'John')))[0].value.name]"; expected = 'John' }
    @{ expression = "[items(createObject('list', createArray('a','b','c')))[0].value[1]]"; expected = 'b' }
    @{ expression = "[length(items(createObject('obj', createObject('x', 1, 'y', 2)))[0].value)]"; expected = 2 }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'objectKeys function returns array of keys: <expression>' -TestCases @(
    @{ expression = "[length(objectKeys(createObject('a', 1, 'b', 2, 'c', 3)))]"; expected = 3 }
    @{ expression = "[length(objectKeys(createObject()))]"; expected = 0 }
    @{ expression = "[objectKeys(createObject('name', 'John'))[0]]"; expected = 'name' }
    @{ expression = "[length(objectKeys(createObject('x', 1, 'y', 2)))]"; expected = 2 }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'objectKeys function works with nested objects: <expression>' -TestCases @(
    @{ expression = "[objectKeys(createObject('person', createObject('name', 'John')))[0]]"; expected = 'person' }
    @{ expression = "[length(objectKeys(createObject('a', createArray(1,2,3), 'b', 'text')))]"; expected = 2 }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'shallowMerge function basic usage: <expression>' -TestCases @(
    @{ expression = "[shallowMerge(createArray(createObject('one', 'a'), createObject('two', 'b'), createObject('two', 'c')))]"; expected = [pscustomobject]@{ one = 'a'; two = 'c' } }
    @{ expression = "[shallowMerge(createArray(createObject('a', 1, 'b', 2), createObject('b', 3, 'c', 4)))]"; expected = [pscustomobject]@{ a = 1; b = 3; c = 4 } }
    @{ expression = "[shallowMerge(createArray(createObject('name', 'John', 'age', 30)))]"; expected = [pscustomobject]@{ name = 'John'; age = 30 } }
    @{ expression = "[shallowMerge(createArray())]"; expected = [pscustomobject]@{} }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $result = $out.results[0].result.actualState.output
    
    if ($expected -is [PSCustomObject]) {
      $expectedHash = @{}
      $expected.PSObject.Properties | ForEach-Object { $expectedHash[$_.Name] = $_.Value }
      
      $resultHash = @{}
      $result.PSObject.Properties | ForEach-Object { $resultHash[$_.Name] = $_.Value }
      
      $resultHash.Count | Should -Be $expectedHash.Count
      foreach ($key in $expectedHash.Keys) {
        $resultHash[$key] | Should -Be $expectedHash[$key]
      }
    }
  }

  It 'shallowMerge function with nested objects: <expression>' -TestCases @(
    @{ expression = "[shallowMerge(createArray(createObject('one', 'a', 'nested', createObject('a', 1, 'nested', createObject('c', 3))), createObject('two', 'b', 'nested', createObject('b', 2))))]"; expectedKeys = @('one', 'two', 'nested'); nestedKeys = @('b') }
    @{ expression = "[shallowMerge(createArray(createObject('nested', createObject('x', 1, 'y', 2)), createObject('nested', createObject('z', 3))))]"; expectedKeys = @('nested'); nestedKeys = @('z') }
  ) {
    param($expression, $expectedKeys, $nestedKeys)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $result = $out.results[0].result.actualState.output
    
    # Verify top-level keys
    $result.PSObject.Properties.Name | Should -HaveCount $expectedKeys.Count
    foreach ($key in $expectedKeys) {
      $result.PSObject.Properties.Name | Should -Contain $key
    }
    
    # Verify nested object was completely replaced (shallow merge)
    if ($nestedKeys.Count -gt 0) {
      $result.nested.PSObject.Properties.Name | Should -HaveCount $nestedKeys.Count
      foreach ($key in $nestedKeys) {
        $result.nested.PSObject.Properties.Name | Should -Contain $key
      }
    }
  }

  It 'shallowMerge function with multiple objects: <expression>' -TestCases @(
    @{ expression = "[shallowMerge(createArray(createObject('a', 1), createObject('b', 2), createObject('c', 3), createObject('d', 4)))]"; expectedKeys = @('a', 'b', 'c', 'd') }
    @{ expression = "[length(objectKeys(shallowMerge(createArray(createObject('x', 10), createObject('y', 20)))))]"; expected = 2 }
  ) {
    param($expression, $expectedKeys, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $result = $out.results[0].result.actualState.output
    
    if ($expectedKeys) {
      $result.PSObject.Properties.Name | Should -HaveCount $expectedKeys.Count
      foreach ($key in $expectedKeys) {
        $result.PSObject.Properties.Name | Should -Contain $key
      }
    }
    
    if ($expected) {
      $result | Should -Be $expected
    }
  }

  It 'tryGet() function works for: <expression>' -TestCases @(
    @{ expression = "[tryGet(createObject('a', 1, 'b', 2), 'a')]"; expected = 1 }
    @{ expression = "[tryGet(createObject('a', 1, 'b', 2), 'c')]"; expected = $null }
    @{ expression = "[tryGet(createObject('key', 'value'), 'key')]"; expected = 'value' }
    @{ expression = "[tryGet(createObject('nested', createObject('x', 10)), 'nested')]"; expected = [pscustomobject]@{ x = 10 } }
    @{ expression = "[tryGet(createObject('nested', createObject('x', 10)), 'missing')]"; expected = $null }
    @{ expression = "[tryGet(createArray(1,2,3), 0)]"; expected = 1 }
    @{ expression = "[tryGet(createArray(1,2,3), 3)]"; expected = $null }
    @{ expression = "[tryGet(createArray(1,2,3), -3)]"; expected = $null }
  ) {
    param($expression, $expected)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    if ($expected -is [pscustomobject]) {
      ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
    } else {
      $out.results[0].result.actualState.output | Should -BeExactly $expected
    }
  }

  It 'tryIndexFromEnd() function works for: <expression>' -TestCases @(
    @{ expression = "[tryIndexFromEnd(createArray('a', 'b', 'c'), 1)]"; expected = 'c' }
    @{ expression = "[tryIndexFromEnd(createArray('a', 'b', 'c'), 2)]"; expected = 'b' }
    @{ expression = "[tryIndexFromEnd(createArray('a', 'b', 'c'), 3)]"; expected = 'a' }
    @{ expression = "[tryIndexFromEnd(createArray('a', 'b', 'c'), 4)]"; expected = $null }
    @{ expression = "[tryIndexFromEnd(createArray('a', 'b', 'c'), 0)]"; expected = $null }
    @{ expression = "[tryIndexFromEnd(createArray('a', 'b', 'c'), -1)]"; expected = $null }
    @{ expression = "[tryIndexFromEnd(createArray('only'), 1)]"; expected = 'only' }
    @{ expression = "[tryIndexFromEnd(createArray(10, 20, 30, 40), 2)]"; expected = 30 }
    @{ expression = "[tryIndexFromEnd(createArray(createObject('k', 'v1'), createObject('k', 'v2')), 1)]"; expected = [pscustomobject]@{ k = 'v2' } }
    @{ expression = "[tryIndexFromEnd(createArray(createArray(1, 2), createArray(3, 4)), 1)]"; expected = @(3, 4) }
    @{ expression = "[tryIndexFromEnd(createArray(), 1)]"; expected = $null }
    @{ expression = "[tryIndexFromEnd(createArray('x', 'y'), 1000)]"; expected = $null }
  ) {
    param($expression, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    if ($expected -is [pscustomobject]) {
      ($out.results[0].result.actualState.output | Out-String) | Should -BeExactly ($expected | Out-String)
    } elseif ($expected -is [array]) {
      ($out.results[0].result.actualState.output | ConvertTo-Json -Compress) | Should -BeExactly ($expected | ConvertTo-Json -Compress)
    } else {
      $out.results[0].result.actualState.output | Should -BeExactly $expected
    }
  }

  It 'uriComponent function works for: <testInput>' -TestCases @(
    @{ testInput = 'hello world' }
    @{ testInput = 'hello@example.com' }
    @{ testInput = 'https://example.com/path?query=value' }
    @{ testInput = '' }
    @{ testInput = 'ABCabc123-_.~' }
    @{ testInput = ':/?#[]@!$&()*+,;=' }
    @{ testInput = 'café' }
    @{ testInput = 'name=John Doe&age=30' }
    @{ testInput = '/path/to/my file.txt' }
    @{ testInput = 'user+tag@example.com' }
    @{ testInput = '1234567890' }
    @{ testInput = '100%' }
    @{ testInput = ' ' }
  ) {
    param($testInput)

    $expected = [Uri]::EscapeDataString($testInput)
    $expression = "[uriComponent('$($testInput -replace "'", "''")')]"

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'uriComponent function works with concat' {
    $input1 = 'hello'
    $input2 = ' '
    $input3 = 'world'
    $expected = [Uri]::EscapeDataString($input1 + $input2 + $input3)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[uriComponent(concat('hello', ' ', 'world'))]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'uriComponentToString function works for: <testInput>' -TestCases @(
    @{ testInput = 'hello%20world' }
    @{ testInput = 'hello%40example.com' }
    @{ testInput = 'https%3A%2F%2Fexample.com%2Fpath%3Fquery%3Dvalue' }
    @{ testInput = '' }
    @{ testInput = 'ABCabc123-_.~' }
    @{ testInput = '%3A%2F%3F%23%5B%5D%40%21%24%26%28%29%2A%2B%2C%3B%3D' }
    @{ testInput = 'caf%C3%A9' }
    @{ testInput = 'name%3DJohn%20Doe%26age%3D30' }
    @{ testInput = '%2Fpath%2Fto%2Fmy%20file.txt' }
    @{ testInput = '100%25' }
  ) {
    param($testInput)

    $expected = [Uri]::UnescapeDataString($testInput)
    $expression = "[uriComponentToString('$($testInput -replace "'", "''")')]"

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'uriComponentToString function works with round-trip encoding' {
    $original = 'hello world'
    $expected = $original

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[uriComponentToString(uriComponent('hello world'))]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'uriComponentToString function works with nested round-trip' {
    $original = 'user+tag@example.com'
    $expected = $original

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[uriComponentToString(uriComponent('user+tag@example.com'))]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'uriComponentToString function works with concat' {
    $input1 = 'hello'
    $input2 = '%20'
    $input3 = 'world'
    $expected = [Uri]::UnescapeDataString($input1 + $input2 + $input3)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[uriComponentToString(concat('hello', '%20', 'world'))]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'json() works: <accessor>' -TestCases @(
    @{ data = @{ name = 'John'; age = 30 }; accessor = '.name'; expected = 'John' }
    @{ data = @{ name = 'John'; age = 30 }; accessor = '.age'; expected = 30 }
    @{ data = @(1,2,3); accessor = '[0]'; expected = 1 }
    @{ data = @(1,2,3); accessor = '[2]'; expected = 3 }
    @{ data = 'hello'; accessor = ''; expected = 'hello' }
    @{ data = 42; accessor = ''; expected = 42 }
    @{ data = $true; accessor = ''; expected = $true }
    @{ data = $false; accessor = ''; expected = $false }
    @{ data = $null; accessor = ''; expected = $null }
    @{ data = @{ users = @( @{ name = 'Alice' }, @{ name = 'Bob' } ) }; accessor = '.users[0].name'; expected = 'Alice' }
    @{ data = @{ users = @( @{ name = 'Alice' }, @{ name = 'Bob' } ) }; accessor = '.users[1].name'; expected = 'Bob' }
    @{ data = @{ key = 'value' }; accessor = '.key'; expected = 'value' }
    @{ data = @{ nested = @{ value = 123 } }; accessor = '.nested.value'; expected = 123 }
  ) {
    param($data, $accessor, $expected)

    $jsonString = ConvertTo-Json -Compress -InputObject $data
    $expression = "[json(''$($jsonString)'')$accessor]"

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$expression'
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -Be $expected
  }

  It 'json() error handling: <expression>' -TestCases @(
    @{ expression = "[json('not valid json')]" }
    @{ expression = "[json('{""key"":""value""')]" }
    @{ expression = "[json('')]" }
    @{ expression = "[json('{incomplete')]" }
    @{ expression = "[json('[1,2,')]" }
  ) {
    param($expression)

    $escapedExpression = $expression -replace "'", "''"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: '$escapedExpression'
"@
    $null = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log
    $LASTEXITCODE | Should -Not -Be 0
    $errorContent = Get-Content $TestDrive/error.log -Raw
    $errorContent | Should -Match ([regex]::Escape('Invalid JSON string'))
  }

  It 'uri() function works for: <base> + <relative>' -TestCases @(
    @{ base = 'https://example.com/'; relative = 'path/file.html'; expected = 'https://example.com/path/file.html' }
    @{ base = 'https://example.com/'; relative = '/path/file.html'; expected = 'https://example.com/path/file.html' }
    @{ base = 'https://example.com/api/v1'; relative = 'users'; expected = 'https://example.com/api/users' }
    @{ base = 'https://example.com/api/v1'; relative = '/users'; expected = 'https://example.com/users' }
    @{ base = 'https://example.com'; relative = 'path'; expected = 'https://example.com/path' }
    @{ base = 'https://example.com'; relative = '/path'; expected = 'https://example.com/path' }
    @{ base = 'https://api.example.com/v2/resource/'; relative = 'item/123'; expected = 'https://api.example.com/v2/resource/item/123' }
    @{ base = 'https://example.com/a/b/c/'; relative = 'd/e/f'; expected = 'https://example.com/a/b/c/d/e/f' }
    @{ base = 'https://example.com/old/path'; relative = 'new'; expected = 'https://example.com/old/new' }
    @{ base = 'https://example.com/api/'; relative = 'search?q=test'; expected = 'https://example.com/api/search?q=test' }
    @{ base = 'https://example.com/page'; relative = '#section'; expected = 'https://example.com/page#section' }
    @{ base = 'https://example.com/page'; relative = '?query=value#section'; expected = 'https://example.com/page?query=value#section' }
    @{ base = 'https://example.com/'; relative = ''; expected = 'https://example.com/' }
    @{ base = 'http://example.com/'; relative = 'page.html'; expected = 'http://example.com/page.html' }
    @{ base = 'ftp://example.com/'; relative = 'file.txt'; expected = 'ftp://example.com/file.txt' }
    @{ base = 'file:///C:/path/'; relative = 'file.txt'; expected = 'file:///C:/path/file.txt' }
    @{ base = 'https://example.com:8080/'; relative = 'api'; expected = 'https://example.com:8080/api' }
    @{ base = 'https://example.com:8080/api'; relative = '/v2'; expected = 'https://example.com:8080/v2' }
    @{ base = 'https://example.com/'; relative = 'path'; expected = 'https://example.com/path' }
    @{ base = 'https://example.com/path/'; relative = 'file%20name.txt'; expected = 'https://example.com/path/file%20name.txt' }
    @{ base = 'https://example.com/'; relative = 'path with spaces'; expected = 'https://example.com/path%20with%20spaces' }
    @{ base = 'https://example.com/'; relative = 'path/../other'; expected = 'https://example.com/other' }
    @{ base = 'https://example.com/a/b/'; relative = '../c'; expected = 'https://example.com/a/c' }
    @{ base = 'https://example.com/a/b/'; relative = './c'; expected = 'https://example.com/a/b/c' }
    @{ base = 'https://example.com/path'; relative = 'https://other.com/other'; expected = 'https://other.com/other' }
    @{ base = 'https://example.com/path'; relative = 'http://different.com/path'; expected = 'http://different.com/path' }
    @{ base = 'https://user:pass@example.com/'; relative = 'path'; expected = 'https://user:pass@example.com/path' }
    @{ base = 'https://example.com/'; relative = 'café/file.txt'; expected = 'https://example.com/caf%C3%A9/file.txt' }
    @{ base = 'https://[::1]/'; relative = 'path'; expected = 'https://[::1]/path' }
    @{ base = 'https://[2001:db8::1]/'; relative = 'api/v1'; expected = 'https://[2001:db8::1]/api/v1' }
    @{ base = 'https://[2001:db8::1]:8080/'; relative = 'api'; expected = 'https://[2001:db8::1]:8080/api' }
    @{ base = 'http://192.168.1.1/'; relative = 'api/v1'; expected = 'http://192.168.1.1/api/v1' }
  ) {
    param($base, $relative, $expected)

    $expression = "[uri('$($base -replace "'", "''")','$($relative -replace "'", "''")')]"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'uri() error handling: <expectedError>' -TestCases @(
    @{ base = ''; relative = 'path'; expectedError = 'The baseUri parameter cannot be empty' }
    @{ base = 'example.com'; relative = 'path'; expectedError = 'The baseUri must be an absolute URI (must include a scheme such as https:// or file://)' }
    @{ base = '/relative/path'; relative = 'file.txt'; expectedError = 'The baseUri must be an absolute URI' }
    @{ base = 'https://example.com/'; relative = '///foo'; expectedError = 'Invalid URI: The relative URI contains an invalid sequence.' }
  ) {
    param($base, $relative, $expectedError)

    $expression = "[uri('$($base -replace "'", "''")','$($relative -replace "'", "''")')]"
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $null = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log
    $LASTEXITCODE | Should -Not -Be 0
    $errorContent = Get-Content $TestDrive/error.log -Raw
    $errorContent | Should -Match ([regex]::Escape($expectedError))
  }

  It 'resourceId allows for arbitrary characters in names including unicode' {
    $name = 'My Resource @123/!#$%^&*()[]{}-+=;`~'
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: "$name"
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[resourceId('Microsoft.DSC.Debug/Echo', '$name')]"
"@
    $out = dsc config get -i $config_yaml | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $expected = "Microsoft.DSC.Debug/Echo:$([Uri]::EscapeDataString($name))"
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'parseCidr parses IPv4 CIDR notation: <cidr>' -TestCases @(
    @{ cidr = '192.168.1.0/24'; network = '192.168.1.0'; broadcast = '192.168.1.255'; firstUsable = '192.168.1.1'; lastUsable = '192.168.1.254'; netmask = '255.255.255.0'; prefix = 24 }
    @{ cidr = '10.0.0.0/16'; network = '10.0.0.0'; broadcast = '10.0.255.255'; firstUsable = '10.0.0.1'; lastUsable = '10.0.255.254'; netmask = '255.255.0.0'; prefix = 16 }
    @{ cidr = '10.144.0.0/20'; network = '10.144.0.0'; broadcast = '10.144.15.255'; firstUsable = '10.144.0.1'; lastUsable = '10.144.15.254'; netmask = '255.255.240.0'; prefix = 20 }
    @{ cidr = '172.16.0.0/12'; network = '172.16.0.0'; broadcast = '172.31.255.255'; firstUsable = '172.16.0.1'; lastUsable = '172.31.255.254'; netmask = '255.240.0.0'; prefix = 12 }
    @{ cidr = '192.168.1.100/32'; network = '192.168.1.100'; broadcast = '192.168.1.100'; firstUsable = '192.168.1.100'; lastUsable = '192.168.1.100'; netmask = '255.255.255.255'; prefix = 32 }
  ) {
    param($cidr, $network, $broadcast, $firstUsable, $lastUsable, $netmask, $prefix)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parseCidr('$cidr')]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $result = $out.results[0].result.actualState.output
    $result.network | Should -BeExactly $network
    $result.netmask | Should -BeExactly $netmask
    $result.broadcast | Should -BeExactly $broadcast
    $result.firstUsable | Should -BeExactly $firstUsable
    $result.lastUsable | Should -BeExactly $lastUsable
    $result.cidr | Should -Be $prefix
  }

  It 'parseCidr parses IPv6 CIDR notation: <cidr>' -TestCases @(
    @{ cidr = '2001:db8::/32'; network = '2001:db8::'; prefix = 32 }
    @{ cidr = 'fe80::/64'; network = 'fe80::'; prefix = 64 }
    @{ cidr = '2001:db8::1/128'; network = '2001:db8::1'; prefix = 128 }
  ) {
    param($cidr, $network, $prefix)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parseCidr('$cidr')]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $result = $out.results[0].result.actualState.output
    $result.network | Should -BeExactly $network
    $result.cidr | Should -Be $prefix
    $result.netmask | Should -Not -BeNullOrEmpty
    $result.firstUsable | Should -Not -BeNullOrEmpty
    $result.lastUsable | Should -Not -BeNullOrEmpty
  }

  It 'parseCidr handles CIDR with host bits set' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parseCidr('192.168.1.100/24')]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $result = $out.results[0].result.actualState.output
    $result.network | Should -BeExactly '192.168.1.0'
    $result.broadcast | Should -BeExactly '192.168.1.255'
  }

  It 'parseCidr fails with invalid CIDR: <cidr>' -TestCases @(
    @{ cidr = '192.168.1/24'; errorMatch = 'Invalid CIDR notation' }
    @{ cidr = '192.168.1.0/33'; errorMatch = 'Invalid CIDR notation' }
    @{ cidr = '192.168.1.256/24'; errorMatch = 'Invalid CIDR notation' }
  ) {
    param($cidr, $errorMatch)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parseCidr('$cidr')]"
"@
    $errorContent = & { $config_yaml | dsc config get -f - 2>&1 } | Out-String
    $LASTEXITCODE | Should -Be 2
    $errorContent | Should -Match $errorMatch
  }

  It 'cidrSubnet splits IPv4 network into subnets: <network>/<newCidr> index <index>' -TestCases @(
    @{ network = '10.144.0.0/20'; newCidr = 24; index = 0; expected = '10.144.0.0/24' }
    @{ network = '10.144.0.0/20'; newCidr = 24; index = 1; expected = '10.144.1.0/24' }
    @{ network = '10.144.0.0/20'; newCidr = 24; index = 2; expected = '10.144.2.0/24' }
    @{ network = '10.144.0.0/20'; newCidr = 24; index = 3; expected = '10.144.3.0/24' }
    @{ network = '10.144.0.0/20'; newCidr = 24; index = 4; expected = '10.144.4.0/24' }
    @{ network = '10.144.0.0/20'; newCidr = 24; index = 15; expected = '10.144.15.0/24' }
    @{ network = '10.0.0.0/16'; newCidr = 18; index = 0; expected = '10.0.0.0/18' }
    @{ network = '10.0.0.0/16'; newCidr = 18; index = 1; expected = '10.0.64.0/18' }
    @{ network = '192.168.0.0/24'; newCidr = 28; index = 0; expected = '192.168.0.0/28' }
  ) {
    param($network, $newCidr, $index, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrSubnet('$network', $newCidr, $index)]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'cidrSubnet splits IPv6 network into subnets' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrSubnet('2001:db8::/32', 48, 0)]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.output | Should -BeExactly '2001:db8::/48'
  }

  It 'cidrSubnet with same prefix returns original network' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrSubnet('10.144.0.0/20', 20, 0)]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.output | Should -BeExactly '10.144.0.0/20'
  }

  It 'cidrSubnet fails with invalid parameters: <testName>' -TestCases @(
    @{ testName = 'index out of range'; network = '10.144.0.0/20'; newCidr = 24; index = 16; errorMatch = 'out of range' }
    @{ testName = 'negative index'; network = '10.144.0.0/20'; newCidr = 24; index = -1; errorMatch = 'negative' }
    @{ testName = 'new CIDR too small'; network = '10.144.0.0/20'; newCidr = 16; index = 0; errorMatch = 'equal to or larger' }
    @{ testName = 'invalid IPv4 prefix'; network = '10.144.0.0/20'; newCidr = 33; index = 0; errorMatch = 'Invalid IPv4 prefix' }
    @{ testName = 'invalid IPv6 prefix'; network = '2001:db8::/32'; newCidr = 129; index = 0; errorMatch = 'Invalid IPv6 prefix' }
    @{ testName = 'invalid CIDR format'; network = '10.0.0/16'; newCidr = 24; index = 0; errorMatch = 'Invalid CIDR notation' }
  ) {
    param($testName, $network, $newCidr, $index, $errorMatch)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrSubnet('$network', $newCidr, $index)]"
"@
    $errorContent = & { $config_yaml | dsc config get -f - 2>&1 } | Out-String
    $LASTEXITCODE | Should -Be 2
    $errorContent | Should -Match $errorMatch
  }

  It 'cidrSubnet can be used with parseCidr' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[parseCidr(cidrSubnet('10.144.0.0/20', 24, 2))]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $result = $out.results[0].result.actualState.output
    $result.network | Should -BeExactly '10.144.2.0'
    $result.cidr | Should -Be 24
  }

  It 'cidrHost returns usable host IP: <network> index <index>' -TestCases @(
    @{ network = '192.168.1.0/24'; index = 0; expected = '192.168.1.1' }
    @{ network = '192.168.1.0/24'; index = 1; expected = '192.168.1.2' }
    @{ network = '192.168.1.0/24'; index = 10; expected = '192.168.1.11' }
    @{ network = '192.168.1.0/24'; index = 253; expected = '192.168.1.254' }
    @{ network = '10.0.0.0/16'; index = 0; expected = '10.0.0.1' }
    @{ network = '10.0.0.0/16'; index = 99; expected = '10.0.0.100' }
    @{ network = '10.0.0.0/16'; index = 255; expected = '10.0.1.0' }
  ) {
    param($network, $index, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrHost('$network', $index)]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'cidrHost handles /31 point-to-point: index <index>' -TestCases @(
    @{ network = '192.168.1.0/31'; index = 0; expected = '192.168.1.0' }
    @{ network = '192.168.1.0/31'; index = 1; expected = '192.168.1.1' }
  ) {
    param($network, $index, $expected)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrHost('$network', $index)]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.output | Should -BeExactly $expected
  }

  It 'cidrHost works with IPv6' {
    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrHost('2001:db8::/64', 0)]"
"@
    $out = $config_yaml | dsc config get -f - | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0
    $out.results[0].result.actualState.output | Should -BeExactly '2001:db8::1'
  }

  It 'cidrHost fails with invalid parameters: <testName>' -TestCases @(
    @{ testName = '/32 has no usable hosts'; network = '192.168.1.1/32'; index = 0; errorMatch = 'no usable host' }
    @{ testName = '/128 has no usable hosts'; network = '2001:db8::1/128'; index = 0; errorMatch = 'no usable host' }
    @{ testName = 'index out of range'; network = '192.168.1.0/24'; index = 254; errorMatch = 'out of range' }
    @{ testName = 'negative index'; network = '192.168.1.0/24'; index = -1; errorMatch = 'negative' }
    @{ testName = 'invalid CIDR'; network = '192.168.1.0.0/24'; index = 0; errorMatch = 'Invalid CIDR notation' }
  ) {
    param($testName, $network, $index, $errorMatch)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[cidrHost('$network', $index)]"
"@
    $errorContent = & { $config_yaml | dsc config get -f - 2>&1 } | Out-String
    $LASTEXITCODE | Should -Be 2
    $errorContent | Should -Match $errorMatch
  }

  It 'tryWhich() works for: <expression>' -TestCases @(
    @{ expression = "[tryWhich('pwsh')]"; found = $true }
    @{ expression = "[tryWhich('nonexistentcommand12345')]"; found = $false }
  ) {
    param($expression, $found)

    $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "$expression"
"@
    $out = dsc -l trace config get -i $config_yaml 2>$TestDrive/error.log | ConvertFrom-Json
    $LASTEXITCODE | Should -Be 0 -Because (Get-Content $TestDrive/error.log -Raw)
    if ($found) {
      $out.results[0].result.actualState.output | Should -Not -BeNullOrEmpty
    } else {
      $out.results[0].result.actualState.output | Should -BeNullOrEmpty
    }
  }
}
