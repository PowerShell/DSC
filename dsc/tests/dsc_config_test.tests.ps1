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
             bitness: 64
         - name: Family Test
           type: Microsoft/OSInfo
           properties:
             family: Windows
'@

        $out = dsc config test -i $configYaml 2> "$TestDrive/trace.log" | ConvertFrom-Json
        if ($IsWindows) {
            $LASTEXITCODE | Should -Be 0
            $out.results[0].result.inDesiredState | Should -BeTrue
        } else {
            $LASTEXITCODE | Should -Be 2
            $log = Get-Content "$TestDrive/trace.log" -Raw
            $log | Should -Match '.*Assertion failed.*'
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

    It 'Duplicate resource names are not allowed' {
        $configYaml = @'
  $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
  resources:
    - name: MyTest
      type: Microsoft.DSC.Debug/Echo
      properties:
        output: 1
    - name: Test2
      type: Microsoft.DSC.Debug/Echo
      properties:
        output: 2
    - name: MyTest
      type: Microsoft.DSC.Debug/Echo
      properties:
        output: 2
'@

        $null = dsc config test -i $configYaml 2> "$TestDrive/trace.log"
        $LASTEXITCODE | Should -Be 2
        $log = Get-Content "$TestDrive/trace.log" -Raw
        $log | Should -Match ".*Resource named 'MyTest' for type 'Microsoft.DSC.Debug/Echo' is specified more than once.*" -Because ($log | Out-String)
    }
}
