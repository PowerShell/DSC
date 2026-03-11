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

    It 'stateAndDiff resource set returns changed properties when not in desired state' {
        $result = '{"valueOne":3,"valueTwo":4}' | dsc resource set -r Test/StateAndDiff -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.afterState.valueOne | Should -Be 1
        $result.afterState.valueTwo | Should -Be 2
        $result.changedProperties | Should -Contain 'valueOne'
        $result.changedProperties | Should -Contain 'valueTwo'
    }

    It 'stateAndDiff resource set returns no changed properties when in desired state' {
        $result = '{"valueOne":1,"valueTwo":2}' | dsc resource set -r Test/StateAndDiff -f - | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $result.afterState.valueOne | Should -Be 1
        $result.afterState.valueTwo | Should -Be 2
        $result.changedProperties | Should -BeNullOrEmpty
    }
}
