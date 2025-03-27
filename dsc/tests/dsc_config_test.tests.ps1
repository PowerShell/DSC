# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc config test tests' {
    It 'Assertion works correctly' {
        $configYaml = @'
 $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
 resources:
   - name: Operating System Assertion
     type: Microsoft.DSC/Assertion
     properties:
       $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
       resources:
         - name: Is64BitOS
           type: Microsoft/OSInfo
           properties:
             bitness: '64'
         - name: 64bit test 2
           type: Microsoft/OSInfo
           properties:
             family: Windows
'@

        $out = dsc config test -i $configYaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0

        if ($IsWindows) {
            $out.results[0].result.inDesiredState | Should -BeTrue
        }
        else {
            $out.results[0].result.inDesiredState | Should -BeFalse
            $out.results[0].result.differingProperties | Should -Contain 'resources'
        }
    }

    It '_inDesiredState returned is used when: inDesiredState = <inDesiredState> and same = <same>' -TestCases @(
        @{ inDesiredState = $true; valueOne = 1; valueTwo = 2; same = $true }
        @{ inDesiredState = $true; valueOne = 3; valueTwo = 4; same = $false }
        @{ inDesiredState = $false; valueOne = 1; valueTwo = 2; same = $true }
        @{ inDesiredState = $false; valueOne = 3; valueTwo = 4; same = $false }
    ) {
        param($inDesiredState, $valueOne, $valueTwo)

        $configYaml = @"
  `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
  resources:
    - name: Test
      type: Test/InDesiredState
      properties:
        _inDesiredState: $inDesiredState
        valueOne: $valueOne
        valueTwo: $valueTwo
"@

        $out = dsc config test -i $configYaml | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].result.inDesiredState | Should -Be $inDesiredState
        if ($same) {
            $out.results[0].result.differingProperties | Should -BeNullOrEmpty
        }
        else {
            $out.results[0].result.differingProperties | Should -Be @('valueOne', 'valueTwo')
        }
    }
}
