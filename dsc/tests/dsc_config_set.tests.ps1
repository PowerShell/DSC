# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'dsc config get tests' {
    It 'can use _exist with resources that support and do not support it' {
        $config_yaml = @"
            `$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
            resources:
            - name: Exist
              type: Test/Exist
              properties:
                _exist: false
            - name: Delete
              type: Test/Delete
              properties:
                _exist: false
"@
        $out = $config_yaml | dsc config set | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.results[0].type | Should -BeExactly 'Test/Exist'
        $out.results[0].result.beforeState.state | Should -BeExactly 'Absent'
        $out.results[0].result.beforeState._exist | Should -BeFalse
        $out.results[0].result.afterState.state | Should -BeExactly 'Absent'
        $out.results[0].result.afterState._exist | Should -BeFalse
        $out.results[1].type | Should -BeExactly 'Test/Delete'
        $out.results[1].result.beforeState.deleteCalled | Should -BeTrue
        $out.results[1].result.beforeState._exist | Should -BeFalse
        $out.results[1].result.afterState.deleteCalled | Should -BeTrue
        $out.results[1].result.afterState._exist | Should -BeFalse
    }
}
