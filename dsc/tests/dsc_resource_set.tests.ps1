# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Invoke a resource set directly' {
    It 'set returns proper error code if no input is provided' {
        $out = dsc resource set -r Test/Version 2>&1
        $LASTEXITCODE | Should -Be 1
        $out | Should -BeLike '*ERROR*'
    }

     It 'version works' {
        $out = dsc resource set -r Test/Version --version 1.1.2 --input '{"version":"1.1.2"}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.afterState.version | Should -BeExactly '1.1.2'
        $out.changedProperties | Should -BeNullOrEmpty
    }

    It '_exist false routes to delete operation' {
        $out = dsc -l trace resource set --resource Test/Delete --input '{"_exist": false}' 2>&1
        $LASTEXITCODE | Should -Be 0
        $out | Out-String | Should -Match 'Routing to delete operation because _exist is false'
    }

    It 'what-if execution of WhatIf resource via <alias>' -TestCases @(
        @{ alias = '-w' }
        @{ alias = '--what-if' }
        @{ alias = '--dry-run' }
        @{ alias = '--noop' }
    ) {
        param($alias)
        $result = dsc resource set $alias -r Test/WhatIf --input '{"executionType":"Actual"}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.afterState.executionType | Should -BeExactly 'WhatIf'
        $result.changedProperties | Should -Contain 'executionType'
    }

    It 'actual execution of WhatIf resource' {
        $result = dsc resource set -r Test/WhatIf --input '{"executionType":"Actual"}' | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.afterState.executionType | Should -BeExactly 'Actual'
        $result.changedProperties | Should -Be $null
    }
}
