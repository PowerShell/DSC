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
        @{ format = $null}
        @{ format = "yyyy-MM-dd"}
        @{ format = "yyyy-MM-ddTHH"}
        @{ format = "yyyy-MM-ddTHHZ"}
        @{ format = "MMM dd, yyyy HH"}
        @{ format = "yy-MMMM-dddd tt H" }
        @{ format = "MMM ddd zzz" }
        @{ format = "YY YYYY MM MMM MMMM" }
    ) {
        param($format)

        if ($null -eq $format) {
            $expected = (Get-Date -AsUTC).ToString("o")
        } else {
            $expected = (Get-Date -AsUTC).ToString($format)
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
        # since the datetimes might slightly differ, we remove the seconds and milliseconds
        $expected = $expected -replace ':\d+\.\d+Z$', 'Z'
        $actual = $actual -replace ':\d+\.\d+Z$', 'Z'
        $actual | Should -BeExactly $expected -Because "Expected: '$expected', Actual: '$actual'"
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
        (Get-Content $TestDrive/error.log -Raw) | Should -Match 'utcNow function can only be used as a parameter default'
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
}
