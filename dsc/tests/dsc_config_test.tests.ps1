# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc config test tests' {
    It 'Assertion works correctly' {
        $configYaml = @'
 $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
 resources:
   - name: Operating System Assertion
     type: Microsoft.DSC/Assertion
     properties:
       $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
}
