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
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
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
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
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
            $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
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
}
