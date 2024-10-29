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
        $out = $config_yaml | dsc config get | ConvertFrom-Json
        $out.results[0].result.actualState.output | Should -Be $expected
    }

    It 'mountedpath(<path>) works' -TestCases @(
        @{ path = '' }
        @{ path = "hello$([System.IO.Path]::DirectorySeparatorChar)world" }
    ) {
        param($path)

        $testPath = if ($path.Length -gt 0) {
            $expected = "$PSHOME$([System.IO.Path]::DirectorySeparatorChar)$path"
            "'$($path.replace('\', '\\'))'"
        }
        else {
            $expected = $PSHOME
            $path
        }

        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
            resources:
            - name: Echo
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: "[mountedpath($testPath)]"
"@
        $out = $config_yaml | dsc config --mounted-path $PSHOME get | ConvertFrom-Json
        $out.results[0].result.actualState.output | Should -BeExactly $expected
    }
}
