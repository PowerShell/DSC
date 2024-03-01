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
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Echo
              type: Test/Echo
              properties:
                output: '$escapedText'
"@
        $out = $config_yaml | dsc config get | ConvertFrom-Json
        $out.results[0].result.actualState.output | Should -Be $expected
    }
}
